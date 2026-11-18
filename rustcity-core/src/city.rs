mod civic;
mod ecology;
mod farm;
mod finance;
mod industry;
mod market;
mod persistence;
mod port;
mod rocket;
use crate::buildings::{BuildingId, Buildings};
use crate::decisions::{Decisions, PlacementId, VisitPhase};
use crate::stats::Stats;
use crate::{BuildDenial, BuildingKind};
use std::{collections::BTreeMap, io};
const MIN_ORE_FOR_MINE: i64 = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Grass,
    Desert,
    Water,
    Tree,
    Tree2,
    Tree3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Build(BuildingKind),
    Query,
    Flood,
    Bulldoze,
    Evacuate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionError {
    DecisionTrace(String),
    OutsideMap(Point),
    SpaceOccupied(Point),
    NothingToEvacuate(Point),
    CannotEvacuate(Point),
    BuildDenied(BuildDenial),
    DesertHere(Point),
    RequiresRiver,
    NoOre,
    NonemptyTip(Point),
    IncompleteMonument(Point),
    ProtectedFire(Point),
    OutOfMoney,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapSizeError;
#[derive(Clone, Copy)]
struct Tile {
    altitude:i32, water_alt:i32, lake:bool,
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
}

impl Building {
    fn new(kind: BuildingKind) -> Self {
        Self::installed(kind, 0)
    }
    fn installed(kind: BuildingKind, tech: i32) -> Self {
        use BuildingKind::*;
        let operation = match kind {
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
            transparent: matches!(kind, PowerLine | TrackBridge | RailBridge | RoadBridge),
            operation,
            production: Default::default(),
            kind,
            neighbors: Vec::new(),
            partners: Vec::new(),
            stock: [0; 10],
            saved: None,
            evacuating: false,
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
        if matches!(building.operation,Operation::Farm(_) | Operation::Civic(civic::Civic::Well(_))) {
            let wet_tiles=(point.y..point.y.saturating_add(size))
                .flat_map(|y| (point.x..point.x.saturating_add(size)).map(move |x| Point { x, y }))
                .filter(|&p| self.has_groundwater(p)==Some(true)).count() as i32;
            match &mut building.operation {
                Operation::Farm(farm)=>farm.groundwater=wet_tiles,
                Operation::Civic(civic::Civic::Well(output))=>*output=wet_tiles*crate::rules::civic::WELL_WATER_PER_WET_TILE,
                _=>unreachable!()
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
            self.mark_coverage(point, 9, civic::MARKET_CHECK);
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
            .filter(|_| side >= 3)
            .ok_or(MapSizeError)?;
        let mut tiles = Vec::new();
        tiles
            .try_reserve_exact(area as usize)
            .map_err(|_| MapSizeError)?;
        tiles.resize(
            area as usize,
            Tile {
                altitude:0, water_alt:0, lake:false, diffuses: false,
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
            people_pool: 100,
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

    fn actual_kind(&self, kind: BuildingKind, point: Point) -> BuildingKind {
        use BuildingKind::*;
        let water = self.terrain(point) == Some(Terrain::Water);
        match kind {
            Track | TrackBridge => {
                if water {
                    TrackBridge
                } else {
                    Track
                }
            }
            Road | RoadBridge => {
                if water {
                    RoadBridge
                } else {
                    Road
                }
            }
            Rail | RailBridge => {
                if water {
                    RailBridge
                } else {
                    Rail
                }
            }
            Windmill | WindPower => {
                if self.tech_level >= WindPower.metadata().required_tech {
                    WindPower
                } else {
                    Windmill
                }
            }
            other => other,
        }
    }

    fn quote_build(&self, kind: BuildingKind, point: Point) -> Result<i32, ActionError> {
        use BuildingKind::*;
        let kind = self.actual_kind(kind, point);
        kind.can_build(
            self.tech_level,
            self.balance,
            self.stats.group_count[School.id() as usize],
            self.stats.group_count[University.id() as usize],
        )
        .map_err(ActionError::BuildDenied)?;
        let size = i32::from(kind.metadata().size);
        if point.x >= self.side - size || point.y >= self.side - size {
            return Err(ActionError::OutsideMap(point));
        }
        let mut groundwater = false;
        let mut ore = 0_i64;
        for y in point.y..point.y + size {
            for x in point.x..point.x + size {
                let cell = Point { x, y };
                let tile = self.tiles[(y * self.side + x) as usize];
                let existing = self.building_at(cell).map(|(kind, _)| kind);
                let bridge = matches!(kind, TrackBridge | RoadBridge | RailBridge);
                let transport = bridge || matches!(kind, Track | Road | Rail);
                if (existing.is_some() || tile.ground == Terrain::Water)
                    && !(transport && existing == Some(PowerLine))
                    && !(bridge && tile.ground == Terrain::Water && existing != Some(kind))
                {
                    return Err(ActionError::SpaceOccupied(point));
                }
                groundwater |= tile.groundwater;
                ore += i64::from(tile.ore);
            }
        }
        if matches!(kind, ParkPond | Parkland | Waterwell) && !groundwater {
            return Err(ActionError::DesertHere(point));
        }
        if kind == OreMine && ore < MIN_ORE_FOR_MINE {
            return Err(ActionError::NoOre);
        }
        if kind == Port
            && !(point.y..point.y + size)
                .all(|y| self.tiles[(y * self.side + point.x + size) as usize].river)
        {
            return Err(ActionError::RequiresRiver);
        }
        Ok(kind.cost(self.tech_level))
    }

    /// Quotes an action against the current city without changing it.
    pub fn preview(&self, action: Action, point: Point) -> Result<i32, ActionError> {
        let visible = 1..self.side - 1;
        if !visible.contains(&point.x) || !visible.contains(&point.y) {
            return Err(ActionError::OutsideMap(point));
        }
        let ground = self.tiles[(point.y * self.side + point.x) as usize].ground;
        match action {
            Action::Build(kind) => self.quote_build(kind, point),
            Action::Query => Ok(0),
            Action::Evacuate => match self.building_at(point) {
                None => Err(ActionError::NothingToEvacuate(point)),
                Some((BuildingKind::Monument,_)) if self.buildings.at(point).and_then(|id|self.buildings.get(id)).is_some_and(|(_,b)|matches!(&b.operation,Operation::Civic(civic::Civic::Monument(m)) if m.completed))=> Err(ActionError::CannotEvacuate(point)),
                Some((kind, _))
                    if matches!(
                        kind,
                        BuildingKind::Parkland
                            | BuildingKind::ParkPond
                            | BuildingKind::Road
                            | BuildingKind::Rail
                            | BuildingKind::Track
                            | BuildingKind::RoadBridge
                            | BuildingKind::RailBridge
                            | BuildingKind::TrackBridge
                            | BuildingKind::PowerLine
                            | BuildingKind::Tip
                            | BuildingKind::Shanty
                    ) =>
                {
                    Err(ActionError::CannotEvacuate(point))
                }
                Some(_) => Ok(0),
            },
            Action::Flood if ground == Terrain::Water || self.buildings.at(point).is_some() => {
                Err(ActionError::SpaceOccupied(point))
            }
            Action::Flood => Ok(crate::rules::FLOOD_COST),
            Action::Bulldoze
                if let Some((origin, building)) = self
                    .buildings
                    .at(point)
                    .and_then(|id| self.buildings.get(id))
                    && let Operation::Industry(industry::IndustryOperation::Tip(tip)) =
                        &building.operation
                    && !tip.can_bulldoze() =>
            {
                Err(ActionError::NonemptyTip(origin))
            }
            Action::Bulldoze
                if let Some((origin, b)) = self
                    .buildings
                    .at(point)
                    .and_then(|id| self.buildings.get(id))
                    && let Operation::Civic(c) = &b.operation
                    && c.incomplete_monument() =>
            {
                Err(ActionError::IncompleteMonument(origin))
            }
            Action::Bulldoze
                if let Some((BuildingKind::Fire, origin)) = self.building_at(point) =>
            {
                Err(ActionError::ProtectedFire(origin))
            }
            Action::Bulldoze if self.buildings.at(point).is_some() => Ok(self
                .building_at(point)
                .unwrap()
                .0
                .metadata()
                .demolition_cost),
            Action::Bulldoze => Ok(match ground {
                Terrain::Water => crate::rules::DRAIN_COST,
                Terrain::Desert => 0,
                _ => crate::rules::CLEAR_LAND_COST,
            }),
        }
    }

    /// Revalidates an action and charges its cost before altering the map.
    pub fn apply(&mut self, action: Action, point: Point) -> Result<(), ActionError> {
        let cost = self.preview(action, point)?;
        let index = (point.y * self.side + point.x) as usize;
        let balance = self
            .balance
            .checked_sub(cost)
            .filter(|v| *v >= crate::rules::CREDIT_LIMIT)
            .ok_or(ActionError::OutOfMoney)?;
        if self.balance > 0 && balance <= 0 {
            self.events.push_back(CityEvent::OutOfMoney);
        }
        self.balance = balance;
        self.stats.expenses.construction.acc =
            self.stats.expenses.construction.acc.saturating_add(cost);
        let mut origin = point;
        let mut size = 1;
        match action {
            Action::Query => return Ok(()),
            Action::Evacuate => {
                let building = self
                    .buildings
                    .get_mut(self.buildings.at(point).unwrap())
                    .unwrap();
                building.evacuating = !building.evacuating;
                if let Operation::Market(m) = &mut building.operation {
                    m.evacuate(building.evacuating);
                }
                return Ok(());
            }
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
                            if kind != BuildingKind::PowerLine {
                                tile.ground = Terrain::Desert;
                            } else if tile.ground != Terrain::Desert {
                                tile.ground = Terrain::Grass;
                            }
                            tile.frame = 0;
                        }
                    }
                }
            }
            Action::Flood => {
                self.tiles[index].ground = Terrain::Water;
                self.tiles[index].frame = 0;
                self.tiles[index].groundwater = true;
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
                    self.tiles[index].ground = match self.tiles[index].ground {
                        Terrain::Desert => return Ok(()),
                        Terrain::Water => Terrain::Grass,
                        _ => Terrain::Desert,
                    };
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
mod simulation;
mod trading;
mod transport;
pub use observations::CityReport;
pub use transport::VehicleFrame;
mod environment;
