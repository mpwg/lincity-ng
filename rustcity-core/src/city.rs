use crate::buildings::Buildings;
use crate::stats::Stats;
use crate::{BuildDenial, BuildingKind};
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    OutsideMap(Point),
    SpaceOccupied(Point),
    NothingToEvacuate(Point),
    CannotEvacuate(Point),
    BuildDenied(BuildDenial),
    DesertHere(Point),
    RequiresRiver,
    NoOre,
    OutOfMoney,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapSizeError;
#[derive(Clone, Copy)]
struct Tile {
    river: bool,
    ore: i32,
    ground: Terrain,
    frame: u16,
    groundwater: bool,
}

struct Building {
    stock: [i32; 10],
    saved: Option<crate::save::Element>,
    kind: BuildingKind,
    evacuating: bool,
}

impl Building {
    fn new(kind: BuildingKind) -> Self {
        Self {
            kind,
            stock: [0; 10],
            saved: None,
            evacuating: false,
        }
    }
}

/// An independent city, including its terrain and financial accounts.
pub struct City {
    side: i32,
    tiles: Vec<Tile>,
    buildings: Buildings<Building>,
    balance: i32,
    stats: Stats,
}

impl City {
    /// Creates the empty grass map used as the starting point for loading or generation.
    pub fn empty(side: i32) -> Result<Self, MapSizeError> {
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
                ground: Terrain::Grass,
                frame: 0,
                groundwater: false,
                river: false,
                ore: 0,
            },
        );
        Ok(Self {
            side,
            tiles,
            buildings: Buildings::new(side),
            balance: 0,
            stats: Stats::default(),
        })
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
        if self.tiles[a].ground == self.tiles[b].ground {
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
                if self.stats.tech_level >= WindPower.metadata().required_tech {
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
            self.stats.tech_level,
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
        Ok(kind.cost(self.stats.tech_level))
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
            Action::Flood => Ok(1_000_000),
            Action::Bulldoze if self.buildings.at(point).is_some() => Ok(self
                .building_at(point)
                .unwrap()
                .0
                .metadata()
                .demolition_cost),
            Action::Bulldoze => Ok(match ground {
                Terrain::Water => 1_000_000,
                Terrain::Desert => 0,
                _ => 1,
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
            .filter(|v| *v >= -2_000_000_000)
            .ok_or(ActionError::OutOfMoney)?;
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
                return Ok(());
            }
            Action::Build(kind) => {
                let kind = self.actual_kind(kind, point);
                size = i32::from(kind.metadata().size);
                if let Some(id) = self.buildings.at(point) {
                    let replaced = self.buildings.remove(id).unwrap();
                    self.stats.group_count[replaced.kind.id() as usize] -= 1;
                }
                self.buildings
                    .place(point, size, Building::new(kind))
                    .expect("validated footprint");
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
                    self.buildings.remove(id);
                    self.stats.group_count[kind.id() as usize] -= 1;
                } else {
                    self.tiles[index].ground = match self.tiles[index].ground {
                        Terrain::Desert => return Ok(()),
                        Terrain::Water => Terrain::Grass,
                        _ => Terrain::Desert,
                    };
                }
            }
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
