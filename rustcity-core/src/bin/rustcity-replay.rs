//! Record or replay a LinCity scenario with a version-one decision tape.
//!
//! ```text
//! rustcity-replay record|replay TAPE SCENARIO traffic|reduced|no-traffic DAYS ACTIONS|- OUTPUT_DIR
//! ```
//!
//! Action files contain a relative day, verb, and tile coordinates on each line.
//! Building also requires a numeric group ID. Use `-` for a run without actions.
//! The output directory receives `state.tsv` and gzip saves on days zero, one,
//! the first month boundary, and the final requested day. Invalid invocations
//! exit with status 2. A failed run exits with status 1.
use rustcity_core::decisions::Decisions;
use rustcity_core::{
    Action, ActionError, BuildDenial, BuildingKind, City, CityEvent, Commodity, Point,
    VehicleTraffic,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::{env, fs, io};

const RECORDING_SEED: u64 = 714;
const MILLIS_PER_SIMULATED_DAY: u64 = 1000;
const CHECKPOINT_DAYS: [u32; 3] = [0, 1, rustcity_core::DAYS_PER_MONTH as u32];
const USAGE: &str = "usage: rustcity-replay record|replay TAPE SCENARIO traffic|reduced|no-traffic DAYS ACTIONS|- OUTPUT_DIR";

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

struct Command {
    day: u32,
    action: Option<Action>,
    point: Point,
    verb: String,
}

fn commands(path: &str, days: u32) -> io::Result<Vec<Command>> {
    let mut cmds = Vec::new();
    if path == "-" {
        return Ok(cmds);
    }
    let mut previous = 1;
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|_| invalid(format!("cannot open actions: {path}")))?
        .read_to_string(&mut text)
        .map_err(|_| invalid("error reading actions"))?;
    for (i, line) in text.lines().enumerate() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = text.split_ascii_whitespace().collect();
        let err = || invalid(format!("invalid action or day at line {}", i + 1));
        let [day, verb, x, y, extra @ ..] = fields.as_slice() else {
            return Err(err());
        };
        let day = day.parse().map_err(|_| err())?;
        if day < previous || day > days {
            return Err(err());
        }
        let point = Point {
            x: x.parse().map_err(|_| err())?,
            y: y.parse().map_err(|_| err())?,
        };
        let (action, remaining) = match *verb {
            "build" => {
                let bad_group = || invalid(format!("invalid group at line {}", i + 1));
                let (id, remaining) = extra.split_first().ok_or_else(bad_group)?;
                let id: u16 = id.parse().map_err(|_| bad_group())?;
                if usize::from(id) >= BuildingKind::GROUP_SLOTS {
                    return Err(bad_group());
                }
                (BuildingKind::from_id(id).map(Action::Build), remaining)
            }
            "bulldoze" => (Some(Action::Bulldoze), extra),
            "evacuate" => (Some(Action::Evacuate), extra),
            "query" => (Some(Action::Query), extra),
            "flood" => (Some(Action::Flood), extra),
            unknown => return Err(invalid(format!("unknown action: {unknown}"))),
        };
        if !remaining.is_empty() {
            return Err(invalid(format!("extra input at line {}", i + 1)));
        }
        cmds.push(Command {
            day,
            action,
            point,
            verb: (*verb).into(),
        });
        previous = day;
    }
    Ok(cmds)
}

fn reason(city: &City, action: Action, p: Point, error: &ActionError) -> String {
    use ActionError::*;
    let loc = |p: &Point| format!("x={} y={}", p.x, p.y);
    match error {
        OutsideMap(p) => format!("outside-map {}", loc(p)),
        SpaceOccupied(p) => format!("space-occupied {}", loc(p)),
        NothingToEvacuate(p) => format!("nothing-to-evacuate {}", loc(p)),
        CannotEvacuate(p) => format!(
            "cannot-evacuate {} group={}",
            loc(p),
            city.building_at(*p).unwrap().0.id()
        ),
        DesertHere(p) => format!("desert-here {}", loc(p)),
        RequiresRiver => "port-requires-river".into(),
        IncompleteMonument(p) => format!(
            "incomplete-monument {} group={}",
            loc(p),
            BuildingKind::Monument.id()
        ),
        ProtectedFire(p) => format!(
            "protected-building {} group={}",
            loc(p),
            BuildingKind::Fire.id()
        ),
        NonemptyTip(p) => format!("nonempty-tip {} group={}", loc(p), BuildingKind::Tip.id()),
        NoOre => "no-ore".into(),
        OutOfMoney => "out-of-money credit-exhausted=1".into(),
        BuildDenied(BuildDenial::OutOfMoney) => "out-of-money credit-exhausted=0".into(),
        BuildDenied(BuildDenial::NotEnoughStudents) => "not-enough-students".into(),
        BuildDenied(BuildDenial::NotEnoughTech) => {
            let Action::Build(kind) = action else {
                unreachable!()
            };
            format!(
                "not-enough-tech current={} required={}",
                city.report().tech_level,
                city.building_kind_at_site(kind, p).metadata().required_tech
            )
        }
        DecisionTrace(msg) => msg.clone(),
    }
}

fn execute(city: &mut City, out: &mut impl Write, i: usize, c: &Command) -> io::Result<()> {
    let action = c
        .action
        .ok_or_else(|| invalid("unregistered building group"))?;
    let p = c.point;
    let key = format!("{}\taction[{i}].{}({},{})", c.day, c.verb, p.x, p.y);
    let allowed = city.preview(action, p);
    writeln!(out, "{key}\tallowed\t{}", u8::from(allowed.is_ok()))?;
    if let Err(e) = allowed {
        writeln!(out, "{key}\treason\t{}", reason(city, action, p, &e))?;
    }
    let quote = city.quote(action, p);
    if let Some(quote) = quote {
        writeln!(out, "{key}\tquote\t{quote}")?;
    }
    let before = city.balance();
    let result = match city.apply(action, p) {
        Ok(()) => "ok".into(),
        Err(ActionError::DecisionTrace(msg)) => return Err(invalid(msg)),
        Err(error) => {
            let detail = reason(city, action, p, &error);
            match action {
                Action::Build(kind)
                    if quote.is_some() && !matches!(error, ActionError::OutOfMoney) =>
                {
                    let id = city.building_kind_at_site(kind, p).id();
                    if matches!(error, ActionError::BuildDenied(_)) {
                        format!("cannot-build group={id} reason={detail}")
                    } else {
                        format!(
                            "cannot-build-here x={} y={} group={id} reason={detail}",
                            p.x, p.y
                        )
                    }
                }
                _ => detail,
            }
        }
    };
    writeln!(out, "{key}\tresult\t{result}")?;
    writeln!(out, "{key}\tmoney_delta\t{}", city.balance() - before)
}

type Inventories = BTreeMap<(i32, i32), (BuildingKind, [i32; Commodity::ALL.len()])>;

fn snapshot(
    day: u32,
    city: &City,
    out: &mut impl Write,
    previous: &mut Inventories,
) -> io::Result<()> {
    let r = city.report();
    for (field, value) in [
        ("time", r.day),
        ("money", r.balance),
        ("people_pool", r.people_pool),
        ("housed_acc", r.housed_acc),
        ("population_acc", r.population_acc),
        ("tech", r.tech_level),
    ] {
        writeln!(out, "{day}\tworld\t{field}\t{value}")?;
    }
    let current: Inventories = city
        .building_inventories()
        .map(|(p, k, s)| ((p.y, p.x), (k, *s)))
        .collect();
    for &(y, x) in previous
        .keys()
        .chain(current.keys())
        .collect::<BTreeSet<_>>()
    {
        let now = current.get(&(y, x));
        let was = previous.get(&(y, x));
        if now == was {
            continue;
        }
        let key = format!("{day}\tbuilding({x},{y})");
        let replaced = now.map(|e| e.0) != was.map(|e| e.0);
        if replaced {
            writeln!(
                out,
                "{key}\tgroup\t{}",
                now.map_or(-1, |e| i32::from(e.0.id()))
            )?;
        }
        if let Some((_, stocks)) = now {
            for c in Commodity::ALL {
                let n = stocks[c as usize];
                if replaced || was.map(|e| e.1[c as usize]) != Some(n) {
                    writeln!(out, "{key}\t{}\t{n}", c.save_name())?;
                }
            }
        }
    }
    *previous = current;
    Ok(())
}

fn run([mode, tape, scenario, profile, days, actions, directory]: &[String; 7]) -> io::Result<()> {
    let days = days
        .parse::<i32>()
        .ok()
        .filter(|_| !days.starts_with('+'))
        .and_then(|days| u32::try_from(days).ok())
        .ok_or_else(|| invalid("invalid day count"))?;
    let profile = match profile.as_str() {
        "traffic" => VehicleTraffic::Normal,
        "reduced" => VehicleTraffic::Reduced,
        "no-traffic" => VehicleTraffic::Disabled,
        _ => return Err(invalid("invalid traffic profile")),
    };
    let actions = commands(actions, days)?;
    let dir = Path::new(directory);
    fs::create_dir_all(dir)?;
    let decisions = match mode.as_str() {
        "record" => Decisions::record(RECORDING_SEED, fs::File::create(tape)?)?,
        "replay" => Decisions::replay(BufReader::new(
            fs::File::open(tape).map_err(|_| invalid("cannot open decision tape"))?,
        ))?,
        _ => return Err(invalid("invalid mode")),
    };
    let input = fs::File::open(scenario)
        .map_err(|_| invalid(format!("failed to open file: {scenario}")))?;
    let mut city = City::load_with_decisions(input, decisions)?;
    city.set_vehicle_traffic(profile);
    let mut out = BufWriter::new(fs::File::create(dir.join("state.tsv"))?);
    let mut previous = Inventories::new();
    let mut next = 0;
    for day in 0..=days {
        if day > 0 {
            while next < actions.len() && actions[next].day == day {
                execute(&mut city, &mut out, next, &actions[next])?;
                next += 1;
            }
            city.animate(u64::from(day) * MILLIS_PER_SIMULATED_DAY)?;
            city.tick()?;
            let mut i = 0;
            while let Some(event) = city.pop_event() {
                let message = match event {
                    CityEvent::FireStarted(p, kind) => {
                        format!("fire-started x={} y={} group={}", p.x, p.y, kind.id())
                    }
                    CityEvent::RocketReady(p) => format!("rocket-ready x={} y={}", p.x, p.y),
                    CityEvent::RocketSuccess(p) => format!("rocket-success x={} y={}", p.x, p.y),
                    CityEvent::RocketFailed(p) => format!("rocket-failed x={} y={}", p.x, p.y),
                    CityEvent::RocketWon(p) => format!("rocket-won x={} y={}", p.x, p.y),
                    CityEvent::RocketEvacuated(p) => {
                        format!("rocket-evacuated x={} y={}", p.x, p.y)
                    }
                    CityEvent::OutOfMoney => "out-of-money credit-exhausted=0".into(),
                    CityEvent::SustainableEconomy(time) => {
                        format!("sustainable-economy time={time}")
                    }
                    CityEvent::NoPeopleLeft(time) => format!("no-people-left time={time}"),
                };
                writeln!(out, "{day}\tmessages\t{i}\t{message}")?;
                i += 1;
            }
        }
        snapshot(day, &city, &mut out, &mut previous)?;
        if CHECKPOINT_DAYS.contains(&day) || day == days {
            city.save(fs::File::create(dir.join(format!("day-{day}.gz")))?)?;
        }
    }
    city.finish_decisions()?;
    out.flush()
}

fn main() {
    let Ok(args) = env::args().skip(1).collect::<Vec<_>>().try_into() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };
    if let Err(error) = run(&args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
