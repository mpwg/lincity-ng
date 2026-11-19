mod construction;
pub use construction::ActionQuote;
mod civic;
mod commune;
mod ecology;
mod world_generation;
pub use world_generation::{Climate, GenerationOptions};
mod farm;
mod finance;
mod industry;
mod market;
mod persistence;
mod port;
mod rocket;
pub use rocket::RocketStage;
mod search;
use crate::buildings::{BuildingId, Buildings};
use crate::decisions::{Decisions, PlacementId, VisitPhase};
use crate::stats::Stats;
use crate::{BuildDenial, BuildingKind};
pub use search::Landform;
use std::{collections::BTreeMap, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Terrain {
    Grass = 0,
    Desert = 42,
    Water = 21,
    Tree = 43,
    Tree2 = 44,
    Tree3 = 45,
}
impl Terrain {
    /// LinCity terrain group used by the save format and tile resource registry.
    pub const fn group(self) -> u16 {
        self as u16
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Build(BuildingKind),
    Query,
    Flood,
    Bulldoze,
    Evacuate,
}
/// Reason an action could not be applied at its target.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ActionError {
    #[error("decision trace error: {0}")]
    DecisionTrace(String),
    #[error("outside the playable map at {0:?}")]
    OutsideMap(Point),
    #[error("occupied site at {0:?}")]
    SpaceOccupied(Point),
    #[error("no building to evacuate at {0:?}")]
    NothingToEvacuate(Point),
    #[error("building at {0:?} cannot be evacuated")]
    CannotEvacuate(Point),
    #[error(transparent)]
    BuildDenied(BuildDenial),
    #[error("desert at {0:?}")]
    DesertHere(Point),
    #[error("port requires a river")]
    RequiresRiver,
    #[error("insufficient ore at the site")]
    NoOre,
    #[error("landfill at {0:?} still contains waste")]
    NonemptyTip(Point),
    #[error("unfinished monument at {0:?}")]
    IncompleteMonument(Point),
    #[error("active fire at {0:?}")]
    ProtectedFire(Point),
    #[error("insufficient funds")]
    OutOfMoney,
}
/// The requested map dimensions exceed the supported range or available allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unsupported city map size")]
pub struct MapSizeError;
#[derive(Clone, Copy)]
struct Tile {
    altitude: i32,
    water_alt: i32,
    lake: bool,
    cables: u8,
    diffuses: bool,
    coverage: u32,
    coal: i32,
    pollution: i32,
    river: bool,
    ore: i32,
    ground: Terrain,
    frame: u16,
    groundwater: bool,
}

enum Operation {
    Commune(commune::Commune),
    Rocket(rocket::Rocket),
    Transport(transport::Transport),
    Market(market::Market),
    Farm(farm::Farm),
    Civic(civic::Civic),
    Industry(industry::IndustryOperation),
    Port(port::Port),
    Workshop(crate::workshop::Workshop),
    Solar(crate::energy::SolarPlant),
    Station(crate::energy::Substation),
    Coal(crate::power::CoalPlant),
    Wind(crate::power::WindGenerator),
    Line(crate::power::PowerLine),
    Other,
}

struct Building {
    sprite_resource: &'static str,
    sprite_frame: i32,
    transparent: bool,
    installed_tech: i32,
    neighbors: Vec<crate::buildings::BuildingId>,
    partners: Vec<crate::buildings::BuildingId>,
    operation: Operation,
    production: crate::power::ProductionCounters,
    stock: [i32; 10],
    saved: Option<crate::save::Element>,
    kind: BuildingKind,
    evacuating: bool,
    never_evacuate: bool,
}

impl Building {
    #[cfg(test)]
    fn new(kind: BuildingKind) -> Self {
        Self::installed(kind, 0)
    }
    fn installed(kind: BuildingKind, tech: i32) -> Self {
        use BuildingKind::*;
        let operation = match kind {
            Commune => Operation::Commune(Default::default()),
            Market => Operation::Market(Default::default()),
            RocketPad => Operation::Rocket(Default::default()),
            Port => Operation::Port(Default::default()),
            Blacksmith | Mill | Pottery => Operation::Workshop(Default::default()),
            SolarPower => Operation::Solar(crate::energy::SolarPlant::new(tech)),
            Substation => Operation::Station(Default::default()),
            CoalPower => Operation::Coal(crate::power::CoalPlant::new(tech)),
            Windmill => Operation::Wind(crate::power::WindGenerator::windmill(tech)),
            WindPower => Operation::Wind(crate::power::WindGenerator::wind_power(tech)),
            PowerLine => Operation::Line(Default::default()),
            kind if kind.is_transport() => Operation::Transport(Default::default()),
            _ => civic::Civic::new(kind).map_or(Operation::Other, Operation::Civic),
        };
        Self {
            installed_tech: tech,
            sprite_resource: kind.metadata().resource,
            sprite_frame: 0,
            transparent: matches!(kind, PowerLine | TrackBridge | RailBridge | RoadBridge),
            operation,
            production: Default::default(),
            kind,
            neighbors: Vec::new(),
            partners: Vec::new(),
            stock: [0; 10],
            saved: None,
            evacuating: false,
            never_evacuate: kind.is_transport()
                || matches!(kind, PowerLine | Parkland | ParkPond | Tip | Shanty),
        }
    }
}

/// An ordered simulation notification for the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CityEvent {
    RocketFailed(Point),
    RocketWon(Point),
    RocketEvacuated(Point),
    RocketReady(Point),
    RocketSuccess(Point),
    SustainableEconomy(i32),
    FireStarted(Point, BuildingKind),
    OutOfMoney,
    NoPeopleLeft(i32),
}

/// An independent city, including its terrain and financial accounts.
pub struct City {
    scenario_name: String,
    rockets_launched: i32,
    rockets_successful: i32,
    game_end: bool,
    decisions: Decisions,
    participants: BTreeMap<PlacementId, BuildingId>,
    displaced: BTreeMap<BuildingId, i32>,
    traffic: VehicleTraffic,
    vehicles: Vec<transport::Vehicle>,
    events: std::collections::VecDeque<CityEvent>,
    saved: Option<crate::save::SaveGame>,
    day: i32,
    tech_level: i32,
    people_pool: i32,
    recent_point: Point,
    side: i32,
    tiles: Vec<Tile>,
    buildings: Buildings<Building>,
    economy: finance::Economy,
    wind_expenses: crate::power::WindExpenses,
    balance: i32,
    stats: Stats,
}

/// Traffic density for simulated commuter vehicles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VehicleTraffic {
    Disabled,
    Reduced,
    Normal,
}

impl City {
    pub fn set_vehicle_traffic(&mut self, traffic: VehicleTraffic) {
        self.traffic = traffic;
    }

    pub fn vehicle_traffic(&self) -> VehicleTraffic {
        self.traffic
    }

    /// Flushes a recording or verifies the end of replay.
    pub fn finish_decisions(self) -> io::Result<()> {
        self.decisions.finish()
    }

    fn create_building(&mut self, kind: BuildingKind, tech: i32) -> io::Result<Building> {
        let mut building = Building::installed(kind, tech);
        if kind == BuildingKind::OrganicFarm {
            building.operation = Operation::Farm(farm::Farm::new(&mut self.decisions)?);
        }
        Ok(building)
    }

    fn place_building(
        &mut self,
        point: Point,
        size: i32,
        mut building: Building,
    ) -> io::Result<BuildingId> {
        if matches!(
            building.operation,
            Operation::Farm(_) | Operation::Civic(civic::Civic::Well(_)) | Operation::Commune(_)
        ) {
            let wet_tiles = (point.y..point.y.saturating_add(size))
                .flat_map(|y| (point.x..point.x.saturating_add(size)).map(move |x| Point { x, y }))
                .filter(|&p| self.has_groundwater(p) == Some(true))
                .count() as i32;
            match &mut building.operation {
                Operation::Farm(farm) => farm.groundwater = wet_tiles,
                Operation::Commune(c) => c.wet_tiles = wet_tiles,
                Operation::Civic(civic::Civic::Well(output)) => {
                    output.output = wet_tiles * crate::rules::civic::WELL_WATER_PER_WET_TILE
                }
                _ => unreachable!(),
            }
        }
        let trace_id = self.decisions.placed(point, building.kind.id())?;
        let id = self.buildings.place(point, size, building).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid or overlapping building footprint at {},{}",
                    point.x, point.y
                ),
            )
        })?;
        self.participants.insert(trace_id, id);
        self.install_industry(id, point)?;
        self.connect_building(point);
        if self.buildings.get(id).unwrap().1.kind == BuildingKind::Market {
            self.mark_coverage(
                point,
                BuildingKind::Market.metadata().range,
                civic::MARKET_CHECK,
            );
        }
        Ok(id)
    }

    fn visit_order(&mut self, phase: VisitPhase, time: u64) -> io::Result<Vec<BuildingId>> {
        let mut order: Vec<_> = self.participants.keys().copied().collect();
        self.decisions.visit(phase, time, &mut order)?;
        Ok(order.iter().map(|id| self.participants[id]).collect())
    }

    /// Creates the empty grass map used as the starting point for loading or generation.
    pub fn empty(side: i32) -> Result<Self, MapSizeError> {
        Self::empty_with_decisions(side, Decisions::native(rand::random()))
    }

    pub fn empty_with_decisions(side: i32, decisions: Decisions) -> Result<Self, MapSizeError> {
        let area = side
            .checked_mul(side)
            .filter(|_| side >= crate::rules::MIN_MAP_SIDE)
            .ok_or(MapSizeError)?;
        let mut tiles = Vec::new();
        tiles
            .try_reserve_exact(area as usize)
            .map_err(|_| MapSizeError)?;
        tiles.resize(
            area as usize,
            Tile {
                altitude: 0,
                water_alt: 0,
                lake: false,
                diffuses: false,
                ground: Terrain::Grass,
                frame: 0,
                groundwater: false,
                coverage: 0,
                pollution: 0,
                river: false,
                ore: 0,
                coal: 0,
                cables: 0,
            },
        );
        Ok(Self {
            scenario_name: String::new(),
            decisions,
            rockets_launched: 0,
            rockets_successful: 0,
            game_end: false,
            participants: BTreeMap::new(),
            displaced: BTreeMap::new(),
            traffic: VehicleTraffic::Disabled,
            vehicles: Vec::new(),
            saved: None,
            day: 0,
            tech_level: 0,
            people_pool: crate::rules::INITIAL_PEOPLE_POOL,
            recent_point: Point {
                x: side / 2,
                y: side / 2,
            },
            side,
            tiles,
            events: Default::default(),
            buildings: Buildings::new(side),
            economy: Default::default(),
            wind_expenses: Default::default(),
            balance: 0,
            stats: Stats::default(),
        })
    }

    pub fn pop_event(&mut self) -> Option<CityEvent> {
        self.events.pop_front()
    }

    /// Current treasury balance, including outstanding credit.
    pub fn balance(&self) -> i32 {
        self.balance
    }

    fn index(&self, point: Point) -> Option<usize> {
        if (0..self.side).contains(&point.x) && (0..self.side).contains(&point.y) {
            Some((point.y * self.side + point.x) as usize)
        } else {
            None
        }
    }

    /// Returns the ground at a map coordinate, including the outer border.
    pub fn terrain(&self, point: Point) -> Option<Terrain> {
        self.index(point).map(|index| self.tiles[index].ground)
    }

    /// Connection mask used to choose the terrain sprite.
    pub fn ground_frame(&self, point: Point) -> Option<u16> {
        self.index(point).map(|i| self.tiles[i].frame)
    }
    /// Underground water can remain after a surface water tile is drained.
    pub fn has_groundwater(&self, point: Point) -> Option<bool> {
        self.index(point).map(|i| self.tiles[i].groundwater)
    }

    // Visit the same interfaces as Map::desert_water_frontiers, including
    // unchanged adjacent pairs: their sprite bits may still be uninitialized.
    fn update_frontiers(&mut self, lo: Point, hi: Point) {
        for y in (lo.y - 1).max(1)..hi.y {
            for x in (lo.x - 1).max(1)..hi.x {
                let i = (y * self.side + x) as usize;
                if y >= lo.y && x + 1 < self.side - 1 {
                    self.connect_edge(i, i + 1, 2, 4);
                }
                if x >= lo.x && y + 1 < self.side - 1 {
                    self.connect_edge(i, i + self.side as usize, 1, 8);
                }
            }
        }
    }

    fn connect_edge(&mut self, a: usize, b: usize, mask_a: u16, mask_b: u16) {
        if self.visible_group(a) == self.visible_group(b) {
            self.tiles[a].frame |= mask_a;
            self.tiles[b].frame |= mask_b;
        } else {
            self.tiles[a].frame &= !mask_a;
            self.tiles[b].frame &= !mask_b;
        }
    }

    /// The building covering a cell and its northwestern origin.
    pub fn building_at(&self, point: Point) -> Option<(BuildingKind, Point)> {
        let (origin, building) = self.buildings.get(self.buildings.at(point)?)?;
        Some((building.kind, origin))
    }

    /// Revalidates an action and charges its cost before altering the map.
    pub fn apply(&mut self, action: Action, point: Point) -> Result<(), ActionError> {
        let quote = construction::Planner::new(self).payable_quote(action, point);
        quote.status?;
        self.execute_action(action, point, quote.cost)
    }

    fn execute_action(
        &mut self,
        action: Action,
        point: Point,
        cost: i32,
    ) -> Result<(), ActionError> {
        let index = (point.y * self.side + point.x) as usize;
        if action == Action::Query
            || (action == Action::Bulldoze
                && self.tiles[index].ground == Terrain::Desert
                && self.buildings.at(point).is_none())
        {
            return Ok(());
        }
        if action == Action::Evacuate {
            let building = self
                .buildings
                .get_mut(self.buildings.at(point).unwrap())
                .unwrap();
            building.evacuating = !building.evacuating;
            if let Operation::Market(market) = &mut building.operation {
                market.evacuate(building.evacuating);
            }
            return Ok(());
        }
        if !finance::expense(
            &mut self.balance,
            &mut self.stats.expenses.construction,
            &mut self.events,
            cost,
        ) {
            return Err(ActionError::OutOfMoney);
        }
        let mut origin = point;
        let mut size = 1;
        match action {
            Action::Query | Action::Evacuate => unreachable!("handled before charging"),
            Action::Build(kind) => {
                let kind = self.actual_kind(kind, point);
                size = i32::from(kind.metadata().size);
                if let Some(id) = self.buildings.at(point) {
                    self.disconnect_building(id);
                    let replaced = self.buildings.remove(id).unwrap();
                    self.stats.group_count[replaced.kind.id() as usize] -= 1;
                }
                let building = self
                    .create_building(kind, self.tech_level)
                    .map_err(|e| ActionError::DecisionTrace(e.to_string()))?;
                self.place_building(point, size, building)
                    .map_err(|e| ActionError::DecisionTrace(e.to_string()))?;
                self.recent_point = point;
                self.stats.group_count[kind.id() as usize] += 1;
                for y in point.y..point.y + size {
                    for x in point.x..point.x + size {
                        let tile = &mut self.tiles[(y * self.side + x) as usize];
                        if tile.ground != Terrain::Water {
                            tile.ground = construction::built_ground(kind, tile.ground);
                            tile.frame = 0;
                        }
                    }
                }
            }
            Action::Flood => {
                self.tiles[index].ground = Terrain::Water;
                self.tiles[index].frame = 0;
                self.tiles[index].groundwater = true;
                self.connect_river(point);
            }
            Action::Bulldoze => {
                if let Some(id) = self.buildings.at(point) {
                    let (kind, corner) = self.building_at(point).unwrap();
                    origin = corner;
                    size = i32::from(kind.metadata().size);
                    self.remove_and_restore_site(id, origin);
                    if kind == BuildingKind::Shanty {
                        self.place_fires(origin, size, civic::SHANTY_DEMOLITION_FIRE_AGE)
                            .map_err(|e| ActionError::DecisionTrace(e.to_string()))?;
                    }
                } else {
                    if self.tiles[index].ground == Terrain::Water {
                        self.tiles[index].river = false;
                        self.tiles[index].cables = 0;
                    }
                    self.tiles[index].ground =
                        construction::cleared_ground(self.tiles[index].ground);
                }
            }
        }
        if let Action::Build(_) | Action::Flood = action {
            self.refresh_transport(
                Point {
                    x: origin.x - 2,
                    y: origin.y - 2,
                },
                Point {
                    x: origin.x + size + 1,
                    y: origin.y + size + 1,
                },
            );
        }
        self.update_frontiers(
            origin,
            Point {
                x: origin.x + size,
                y: origin.y + size,
            },
        );
        Ok(())
    }
}
mod observations;
mod report;
mod visuals;
pub use report::{ReportMode, ReportRow};
pub use visuals::{BuildingVisual, SpriteLayer};
mod simulation;
mod trading;
mod transport;
pub use observations::{BuildingState, CityReport, TileView};
pub use transport::VehicleFrame;
mod environment;
