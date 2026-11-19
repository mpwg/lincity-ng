//! Ownership and command boundary between the legacy desktop and RustCity.
use rustcity_core as core;
use std::{fs::File, io, path::Path};
#[cfg(test)]
mod tests;
pub mod view;

#[cxx::bridge(namespace = "rustcity")]
pub mod ffi {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Point {
        x: i32,
        y: i32,
    }
    struct RoadPosition {
        x: f32,
        z: f32,
        heading: i32,
    }
    enum Climate {
        River,
        Desert,
        Temperate,
        Swamp,
    }
    enum Traffic {
        Disabled,
        Reduced,
        Normal,
    }
    enum ActionKind {
        Query,
        Build,
        Bulldoze,
        Evacuate,
        Flood,
    }
    #[derive(Debug)]
    enum Failure {
        None,
        InvalidAction,
        OutsideMap,
        SpaceOccupied,
        NothingToEvacuate,
        CannotEvacuate,
        NoCredit,
        OutOfMoney,
        NotEnoughTech,
        NotEnoughStudents,
        DesertHere,
        RequiresRiver,
        NoOre,
        NonemptyTip,
        IncompleteMonument,
        ProtectedFire,
        DecisionTrace,
    }
    struct ActionResult {
        error: Failure,
        point: Point,
        group: u16,
        cost: i32,
        current_tech: i32,
        required_tech: i32,
    }
    enum EventKind {
        None,
        RocketFailed,
        RocketWon,
        RocketEvacuated,
        RocketReady,
        RocketSuccess,
        SustainableEconomy,
        FireStarted,
        OutOfMoney,
        NoPeopleLeft,
    }
    struct Event {
        kind: EventKind,
        point: Point,
        group: u16,
        value: i32,
    }
    extern "Rust" {
        #[derive(ExternType)]
        type Engine;
        fn generate(side: i32, climate: Climate, village: bool, trees: bool)
        -> Result<Box<Engine>>;
        fn load(path: &str) -> Result<Box<Engine>>;
        fn save(engine: &Engine, path: &str) -> Result<()>;
        fn tick(engine: &mut Engine) -> Result<()>;
        fn advance_animation(engine: &mut Engine, millis: u64) -> Result<Vec<RoadPosition>>;
        fn animate(engine: &mut Engine, millis: u64) -> Result<()>;
        fn set_traffic(engine: &mut Engine, traffic: Traffic) -> Result<()>;
        fn rocket_frame(engine: &Engine, point: Point) -> u8;
        fn factory_operating(engine: &Engine, point: Point) -> bool;
        fn side(engine: &Engine) -> i32;
        fn focus(engine: &Engine) -> Point;
        fn set_focus(engine: &mut Engine, point: Point);
        fn ensure_history(engine: &mut Engine, months: usize);
        fn survey_coal(engine: &mut Engine) -> ActionResult;
        fn market_trade(
            engine: &mut Engine,
            point: Point,
            commodity: u8,
            buy: bool,
            sell: bool,
        ) -> bool;
        fn port_trade(
            engine: &mut Engine,
            point: Point,
            commodity: u8,
            buy: bool,
            sell: bool,
        ) -> bool;
        fn launch(engine: &mut Engine, point: Point) -> bool;
        fn quote(engine: &Engine, kind: ActionKind, group: u16, point: Point) -> ActionResult;
        fn preview(engine: &Engine, kind: ActionKind, group: u16, point: Point) -> ActionResult;
        fn apply(engine: &mut Engine, kind: ActionKind, group: u16, point: Point) -> ActionResult;
        fn can_build(engine: &Engine, group: u16) -> ActionResult;
        fn pop_event(engine: &mut Engine) -> Event;
    }
}

pub struct Engine {
    pub(crate) city: core::City,
}

impl From<core::Point> for ffi::Point {
    fn from(p: core::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}
impl From<ffi::Point> for core::Point {
    fn from(p: ffi::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

fn generate(
    side: i32,
    climate: ffi::Climate,
    village: bool,
    trees: bool,
) -> io::Result<Box<Engine>> {
    let climate = match climate {
        ffi::Climate::River => core::Climate::River,
        ffi::Climate::Desert => core::Climate::Desert,
        ffi::Climate::Temperate => core::Climate::Temperate,
        ffi::Climate::Swamp => core::Climate::Swamp,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown landscape",
            ));
        }
    };
    let city = core::City::generate(
        side,
        core::GenerationOptions {
            climate,
            village,
            trees,
        },
    )?;
    Ok(Box::new(Engine { city }))
}
fn load(path: &str) -> io::Result<Box<Engine>> {
    let city = core::City::load(File::open(path)?)?;
    Ok(Box::new(Engine { city }))
}
fn save(engine: &Engine, path: &str) -> io::Result<()> {
    let path = Path::new(path);
    let directory = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    engine.city.save(temporary.as_file_mut())?;
    temporary.persist(path)?;
    Ok(())
}
fn tick(engine: &mut Engine) -> io::Result<()> {
    engine.city.tick()
}
fn advance_animation(engine: &mut Engine, millis: u64) -> io::Result<Vec<ffi::RoadPosition>> {
    engine.city.animate(millis)?;
    Ok(engine
        .city
        .traffic_positions()
        .map(|(x, z, heading)| ffi::RoadPosition { x, z, heading })
        .collect())
}
fn animate(engine: &mut Engine, millis: u64) -> io::Result<()> {
    engine.city.animate(millis)
}
fn set_traffic(engine: &mut Engine, traffic: ffi::Traffic) -> io::Result<()> {
    engine.city.set_vehicle_traffic(match traffic {
        ffi::Traffic::Disabled => core::VehicleTraffic::Disabled,
        ffi::Traffic::Reduced => core::VehicleTraffic::Reduced,
        ffi::Traffic::Normal => core::VehicleTraffic::Normal,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown traffic setting",
            ));
        }
    });
    Ok(())
}
fn rocket_frame(engine: &Engine, point: ffi::Point) -> u8 {
    engine
        .city
        .building_state(point.into())
        .and_then(|b| b.rocket_frame)
        .unwrap_or(0)
}
fn factory_operating(engine: &Engine, point: ffi::Point) -> bool {
    engine
        .city
        .building_state(point.into())
        .is_some_and(|b| b.industrial_output_today > 0)
}

fn side(engine: &Engine) -> i32 {
    engine.city.side()
}
fn focus(engine: &Engine) -> ffi::Point {
    engine.city.focus().into()
}
fn set_focus(engine: &mut Engine, point: ffi::Point) {
    engine.city.set_focus(point.into());
}
fn ensure_history(engine: &mut Engine, months: usize) {
    engine.city.ensure_population_history(months);
}

fn market_trade(
    engine: &mut Engine,
    point: ffi::Point,
    commodity: u8,
    buy: bool,
    sell: bool,
) -> bool {
    core::Commodity::ALL
        .get(usize::from(commodity))
        .is_some_and(|&c| engine.city.set_market_trade(point.into(), c, buy, sell))
}
fn port_trade(
    engine: &mut Engine,
    point: ffi::Point,
    commodity: u8,
    buy: bool,
    sell: bool,
) -> bool {
    core::Commodity::ALL
        .get(usize::from(commodity))
        .is_some_and(|&c| engine.city.set_port_trade(point.into(), c, buy, sell))
}
fn launch(engine: &mut Engine, point: ffi::Point) -> bool {
    engine.city.launch_rocket(point.into())
}

impl ffi::ActionResult {
    fn new(group: u16, point: ffi::Point) -> Self {
        Self {
            error: ffi::Failure::None,
            point,
            group,
            cost: 0,
            current_tech: 0,
            required_tech: 0,
        }
    }

    fn failed(mut self, error: core::ActionError) -> Self {
        use core::ActionError as E;
        use ffi::Failure as F;
        self.error = match error {
            E::OutsideMap(p) => {
                self.point = p.into();
                F::OutsideMap
            }
            E::SpaceOccupied(p) => {
                self.point = p.into();
                F::SpaceOccupied
            }
            E::NothingToEvacuate(p) => {
                self.point = p.into();
                F::NothingToEvacuate
            }
            E::CannotEvacuate(p) => {
                self.point = p.into();
                F::CannotEvacuate
            }
            E::DesertHere(p) => {
                self.point = p.into();
                F::DesertHere
            }
            E::NonemptyTip(p) => {
                self.point = p.into();
                F::NonemptyTip
            }
            E::IncompleteMonument(p) => {
                self.point = p.into();
                F::IncompleteMonument
            }
            E::ProtectedFire(p) => {
                self.point = p.into();
                F::ProtectedFire
            }
            E::OutOfMoney => F::NoCredit,
            E::RequiresRiver => F::RequiresRiver,
            E::NoOre => F::NoOre,
            E::DecisionTrace(_) => F::DecisionTrace,
            E::BuildDenied(core::BuildDenial::NotEnoughTech) => F::NotEnoughTech,
            E::BuildDenied(core::BuildDenial::NotEnoughStudents) => F::NotEnoughStudents,
            E::BuildDenied(core::BuildDenial::OutOfMoney) => F::OutOfMoney,
        };
        self
    }
}

fn command(
    city: &core::City,
    kind: ffi::ActionKind,
    group: u16,
    point: ffi::Point,
) -> Result<(core::Action, ffi::ActionResult), ffi::ActionResult> {
    use core::Action as A;
    use ffi::ActionKind as K;
    let mut result = ffi::ActionResult::new(group, point);
    let action = match kind {
        K::Build => core::BuildingKind::from_id(group).map(|requested| {
            let actual = city.building_kind_at_site(requested, point.into());
            result.group = actual.id();
            result.current_tech = city.report().tech_level;
            result.required_tech = actual.metadata().required_tech;
            A::Build(requested)
        }),
        K::Query => Some(A::Query),
        K::Bulldoze => Some(A::Bulldoze),
        K::Evacuate => Some(A::Evacuate),
        K::Flood => Some(A::Flood),
        _ => None,
    };
    let Some(action) = action else {
        result.error = ffi::Failure::InvalidAction;
        return Err(result);
    };
    if kind != K::Build {
        result.group = city
            .building_at(point.into())
            .map_or(0, |(kind, _)| kind.id());
    }
    if let Some(cost) = city.quote(action, point.into()) {
        result.cost = cost;
    } else {
        result.error = ffi::Failure::OutsideMap;
        return Err(result);
    }
    Ok((action, result))
}

fn quote(
    engine: &Engine,
    kind: ffi::ActionKind,
    group: u16,
    point: ffi::Point,
) -> ffi::ActionResult {
    match command(&engine.city, kind, group, point) {
        Ok((_, result)) | Err(result) => result,
    }
}
fn preview(
    engine: &Engine,
    kind: ffi::ActionKind,
    group: u16,
    point: ffi::Point,
) -> ffi::ActionResult {
    match command(&engine.city, kind, group, point) {
        Ok((action, result)) => match engine.city.preview(action, point.into()) {
            Ok(_) => result,
            Err(error) => result.failed(error),
        },
        Err(error) => error,
    }
}
fn apply(
    engine: &mut Engine,
    kind: ffi::ActionKind,
    group: u16,
    point: ffi::Point,
) -> ffi::ActionResult {
    match command(&engine.city, kind, group, point) {
        Ok((action, result)) => match engine.city.apply(action, point.into()) {
            Ok(()) => result,
            Err(error) => result.failed(error),
        },
        Err(error) => error,
    }
}
fn can_build(engine: &Engine, group: u16) -> ffi::ActionResult {
    let mut result = ffi::ActionResult::new(group, ffi::Point { x: -1, y: -1 });
    let Some(kind) = core::BuildingKind::from_id(group) else {
        result.error = ffi::Failure::InvalidAction;
        return result;
    };
    let actual = if matches!(
        kind,
        core::BuildingKind::Windmill | core::BuildingKind::WindPower
    ) {
        engine.city.building_kind_at_site(kind, result.point.into())
    } else {
        kind
    };
    result.group = actual.id();
    result.current_tech = engine.city.report().tech_level;
    result.required_tech = actual.metadata().required_tech;
    result.cost = actual.cost(result.current_tech);
    match engine.city.can_build(kind) {
        Ok(()) => result,
        Err(error) => result.failed(core::ActionError::BuildDenied(error)),
    }
}
fn survey_coal(engine: &mut Engine) -> ffi::ActionResult {
    let mut result = ffi::ActionResult::new(0, ffi::Point { x: -1, y: -1 });
    let before = engine.city.balance();
    match engine.city.survey_coal() {
        Ok(()) => {
            result.cost = before - engine.city.balance();
            result
        }
        Err(e) => result.failed(e),
    }
}
fn pop_event(engine: &mut Engine) -> ffi::Event {
    use core::CityEvent as E;
    use ffi::EventKind as K;
    let none = core::Point { x: -1, y: -1 };
    let (kind, point, group, value) = match engine.city.pop_event() {
        None => (K::None, none, 0, 0),
        Some(E::RocketFailed(p)) => (K::RocketFailed, p, 0, 0),
        Some(E::RocketWon(p)) => (K::RocketWon, p, 0, 0),
        Some(E::RocketEvacuated(p)) => (K::RocketEvacuated, p, 0, 0),
        Some(E::RocketReady(p)) => (K::RocketReady, p, 0, 0),
        Some(E::RocketSuccess(p)) => (K::RocketSuccess, p, 0, 0),
        Some(E::SustainableEconomy(day)) => (K::SustainableEconomy, none, 0, day),
        Some(E::FireStarted(p, k)) => (K::FireStarted, p, k.id(), 0),
        Some(E::OutOfMoney) => (K::OutOfMoney, none, 0, 0),
        Some(E::NoPeopleLeft(day)) => (K::NoPeopleLeft, none, 0, day),
    };
    ffi::Event {
        kind,
        point: point.into(),
        group,
        value,
    }
}
