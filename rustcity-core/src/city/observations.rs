//! Read-only views used by simulation hosts and parity tools.
use super::*;
use crate::Commodity;

/// Current world values reported by the decision replay protocol.
#[derive(Clone, Copy, Debug)]
pub struct CityReport {
    pub day: i32,
    pub balance: i32,
    pub people_pool: i32,
    pub housed_acc: i32,
    pub population_acc: i32,
    pub tech_level: i32,
    pub rockets_launched: i32,
    pub successful_launches: i32,
}

/// Read-only state of the selected building, borrowed until the next city mutation.
#[derive(Debug)]
pub struct BuildingState<'city> {
    pub origin: Point,
    /// Stable for the lifetime of this placement in this city.
    pub instance_id: u64,
    pub kind: BuildingKind,
    pub never_evacuate: bool,
    pub occupants: i32,
    /// Goods, steel or coal-fired electricity produced in the most recent tick.
    pub industrial_output_today: i32,
    pub fire_age: Option<(i32, i32)>,
    pub evacuating: bool,
    pub ready_to_launch: bool,
    pub rocket_stage: Option<RocketStage>,
    /// Construction, lift-off or spent-pad frame used by graphical hosts.
    pub rocket_frame: Option<u8>,
    inventory: &'city [i32; Commodity::ALL.len()],
}

impl BuildingState<'_> {
    /// Amount of a commodity currently in storage.
    pub fn stock(&self, commodity: Commodity) -> i32 {
        self.inventory[commodity as usize]
    }
}

/// Terrain, deposits and service coverage at a single map coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileView {
    pub point: Point,
    pub terrain: Terrain,
    pub ground_frame: u16,
    pub altitude: i32,
    pub water_altitude: i32,
    pub lake: bool,
    pub river: bool,
    pub groundwater: bool,
    pub pollution: i32,
    pub coal: i32,
    pub ore: i32,
    pub fire_cover: bool,
    pub health_cover: bool,
    pub sports_cover: bool,
    pub market_cover: bool,
    pub north_south_cable: bool,
    pub east_west_cable: bool,
    /// The occupying building and its northwestern origin.
    pub building: Option<(BuildingKind, Point)>,
}

impl City {
    /// Scenario name saved with this city.
    pub fn scenario_name(&self) -> &str {
        &self.scenario_name
    }

    /// Whether the simulation has issued its end-of-game notification.
    pub fn game_ended(&self) -> bool {
        self.game_end
    }

    /// Saved map position used to center the view when resuming.
    pub fn focus(&self) -> Point {
        self.recent_point
    }

    /// Stores the map position to show when resuming.
    pub fn set_focus(&mut self, point: Point) {
        self.recent_point = point;
    }

    /// Width and height of the square map, including its border.
    pub fn side(&self) -> i32 {
        self.side
    }

    /// Returns a single tile, including the outer border.
    pub fn tile(&self, point: Point) -> Option<TileView> {
        self.index(point)
            .map(|i| self.tile_view(point, &self.tiles[i]))
    }

    fn tile_view(&self, point: Point, tile: &Tile) -> TileView {
        use crate::rules::civic::{CRICKET, FIRE, HEALTH, MARKET_CHECK};
        use crate::rules::transport::{EAST_WEST_CABLE, NORTH_SOUTH_CABLE};
        TileView {
            point,
            terrain: tile.ground,
            ground_frame: tile.frame,
            altitude: tile.altitude,
            water_altitude: tile.water_alt,
            lake: tile.lake,
            river: tile.river,
            groundwater: tile.groundwater,
            pollution: tile.pollution,
            coal: tile.coal,
            ore: tile.ore,
            fire_cover: tile.coverage & FIRE != 0,
            health_cover: tile.coverage & HEALTH != 0,
            sports_cover: tile.coverage & CRICKET != 0,
            market_cover: tile.coverage & (MARKET_CHECK >> 1) != 0,
            north_south_cable: tile.cables & NORTH_SOUTH_CABLE != 0,
            east_west_cable: tile.cables & EAST_WEST_CABLE != 0,
            building: self.building_at(point),
        }
    }

    /// Iterates over a half-open rectangle in row order, clipping it to the map.
    pub fn map_region(
        &self,
        x: std::ops::Range<i32>,
        y: std::ops::Range<i32>,
    ) -> impl Iterator<Item = TileView> + '_ {
        let columns = x.start.max(0)..x.end.min(self.side);
        (y.start.max(0)..y.end.min(self.side)).flat_map(move |y| {
            columns.clone().map(move |x| {
                self.tile_view(Point { x, y }, &self.tiles[(y * self.side + x) as usize])
            })
        })
    }

    /// Returns smoothed commodity flow in parts per thousand of transport capacity.
    /// A missing reading denotes a tile or commodity outside the transport network.
    pub fn commodity_traffic(&self, point: Point, commodity: Commodity) -> Option<i32> {
        let (_, b) = self.buildings.get(self.buildings.at(point)?)?;
        if b.kind.metadata().rule(commodity).max_load == 0 {
            return None;
        }
        match &b.operation {
            Operation::Transport(t) => Some(t.traffic[commodity as usize]),
            Operation::Line(line) => Some(line.traffic),
            _ => None,
        }
    }

    /// Extends the population, unemployment, starvation and homeless histories for charting.
    /// Shorter requests preserve the existing samples.
    pub fn ensure_population_history(&mut self, months: usize) {
        self.stats.ensure_population_history(months);
    }

    /// City accounts including period totals, inventory, demographics and their histories.
    pub fn statistics(&self) -> &crate::stats::Stats {
        &self.stats
    }

    /// Selects the building occupying any square of its footprint.
    pub fn building_state(&self, point: Point) -> Option<BuildingState<'_>> {
        use slotmap::Key;
        let id = self.buildings.at(point)?;
        let (origin, b) = self.buildings.get(id)?;
        Some(BuildingState {
            instance_id: id.data().as_ffi(),
            never_evacuate: b.never_evacuate,
            occupants: match &b.operation {
                Operation::Civic(c) => c.occupants(),
                _ => 0,
            },
            industrial_output_today: match &b.operation {
                Operation::Coal(plant) => plant.generated_today,
                Operation::Industry(industry::IndustryOperation::Light(m)) => m.goods_today,
                Operation::Industry(industry::IndustryOperation::Heavy(m)) => m.steel_today,
                _ => 0,
            },
            fire_age: match &b.operation {
                Operation::Civic(civic::Civic::Fire(f)) => Some((f.burning, f.smoking)),
                _ => None,
            },
            origin,
            kind: b.kind,
            evacuating: b.evacuating,
            inventory: &b.stock,
            rocket_stage: match &b.operation {
                Operation::Rocket(r) => Some(r.stage),
                _ => None,
            },
            rocket_frame: match &b.operation {
                Operation::Rocket(r) => Some(r.visual_frame()),
                _ => None,
            },
            ready_to_launch: matches!(&b.operation, Operation::Rocket(r) if r.ready_to_launch()),
        })
    }

    /// Snapshot of the current day, treasury, population and technology.
    pub fn report(&self) -> CityReport {
        CityReport {
            day: self.day,
            balance: self.balance,
            people_pool: self.people_pool,
            housed_acc: self.stats.population.housed_m.acc,
            population_acc: self.stats.population.population_m.acc,
            tech_level: self.tech_level,
            rockets_launched: self.rockets_launched,
            successful_launches: self.rockets_successful,
        }
    }

    /// Current stock and origin of each live building.
    pub fn building_inventories(&self) -> impl Iterator<Item = (Point, BuildingKind, &[i32; 10])> {
        self.buildings.iter().map(|(p, b)| (p, b.kind, &b.stock))
    }

    /// Resolves alternate groups such as bridges and high-technology wind power.
    pub fn building_kind_at_site(&self, kind: BuildingKind, p: Point) -> BuildingKind {
        self.actual_kind(kind, p)
    }
}
