//! Record or replay a LinCity scenario, emitting daily observations and checkpoint saves.
use rustcity_core::decisions::Decisions;
use rustcity_core::{
    Action, ActionError, BuildDenial, BuildingKind, City, CityEvent, Commodity, Point,
    VehicleTraffic,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::{env, fs, io};

const RECORDING_SEED: u64 = 714;
const MILLIS_PER_SIMULATED_DAY: u64 = 1000;
const CHECKPOINT_DAYS: [u32; 3] = [0, 1, 100];

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

struct Command {
    day: u32,
    action: Action,
    point: Point,
    verb: String,
}

fn commands(path: &str, days: u32) -> io::Result<Vec<Command>> {
    let mut cmds = Vec::new();
    if path == "-" {
        return Ok(cmds);
    }
    let mut previous = 1;
    for (i, line) in fs::read_to_string(path)?.lines().enumerate() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = text.split_ascii_whitespace().collect();
        let err = || invalid(format!("invalid action at line {}", i + 1));
        let [day, verb, x, y, extra @ ..] = fields.as_slice() else {
            return Err(err());
        };
        let day = day.parse().map_err(|_| err())?;
        if day < previous || day > days {
            return Err(err());
        }
        let action = match (*verb, extra) {
            ("build", [id]) => Action::Build(
                BuildingKind::from_id(id.parse().map_err(|_| err())?).ok_or_else(err)?,
            ),
            ("bulldoze", []) => Action::Bulldoze,
            ("evacuate", []) => Action::Evacuate,
            ("query", []) => Action::Query,
            ("flood", []) => Action::Flood,
            _ => return Err(err()),
        };
        let point = Point {
            x: x.parse().map_err(|_| err())?,
            y: y.parse().map_err(|_| err())?,
        };
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
        IncompleteMonument(p) => format!("incomplete-monument {} group=25", loc(p)),
        ProtectedFire(p) => format!("protected-building {} group=34", loc(p)),
        NonemptyTip(p) => format!("nonempty-tip {} group=16", loc(p)),
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
    let p = c.point;
    let key = format!("{}\taction[{i}].{}({},{})", c.day, c.verb, p.x, p.y);
    let allowed = city.preview(c.action, p);
    writeln!(out, "{key}\tallowed\t{}", u8::from(allowed.is_ok()))?;
    if let Err(e) = allowed {
        writeln!(out, "{key}\treason\t{}", reason(city, c.action, p, &e))?;
    }
    let quote = city.quote(c.action, p);
    if let Some(quote) = quote {
        writeln!(out, "{key}\tquote\t{quote}")?;
    }
    let before = city.balance();
    let result = match city.apply(c.action, p) {
        Ok(()) => "ok".into(),
        Err(ActionError::DecisionTrace(msg)) => return Err(invalid(msg)),
        Err(error) => {
            let detail = reason(city, c.action, p, &error);
            match c.action {
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

type Inventories = BTreeMap<(i32, i32), (BuildingKind, [i32; 10])>;

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

fn run(args: &[String]) -> io::Result<()> {
    let [mode, tape, scenario, profile, days, actions, directory] = args else {
        return Err(invalid(
            "usage: rustcity-replay record|replay TAPE SCENARIO traffic|reduced|no-traffic DAYS ACTIONS|- OUTPUT_DIR",
        ));
    };
    let days: u32 = days.parse().map_err(|_| invalid("invalid day count"))?;
    let profile = match profile.as_str() {
        "traffic" => VehicleTraffic::Normal,
        "reduced" => VehicleTraffic::Reduced,
        "no-traffic" => VehicleTraffic::Disabled,
        _ => return Err(invalid("invalid traffic profile")),
    };
    let actions = commands(actions, days)?;
    let decisions = match mode.as_str() {
        "record" => Decisions::record(RECORDING_SEED, fs::File::create(tape)?)?,
        "replay" => Decisions::replay(BufReader::new(fs::File::open(tape)?))?,
        _ => return Err(invalid("invalid replay mode")),
    };
    let mut city = City::load_with_decisions(fs::File::open(scenario)?, decisions)?;
    city.set_vehicle_traffic(profile);
    let dir = Path::new(directory);
    fs::create_dir_all(dir)?;
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
    if let Err(error) = run(&env::args().skip(1).collect::<Vec<_>>()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
