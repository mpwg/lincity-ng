//! The four native terrain generators and the optional starter settlement.
use super::*;
use crate::rules::generation::*;
use crate::rules::industry::{ORE_RESERVE, coal_mine::DEPOSIT_CAPACITY};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
mod elevation;
mod rivers;
mod vegetation;
mod village;

/// Landscape choices exposed by the original city creation menu.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Climate {
    River,
    Desert,
    Temperate,
    Swamp,
}

/// Settings for initial terrain, vegetation and settlement placement.
#[derive(Clone, Copy, Debug)]
pub struct GenerationOptions {
    pub climate: Climate,
    pub village: bool,
    pub trees: bool,
}

impl City {
    /// Creates a new landscape with a fresh, city-local native random source.
    pub fn generate(side: i32, options: GenerationOptions) -> io::Result<Self> {
        Self::generate_with_decisions(side, options, Decisions::native(rand::random()))
    }
    /// Generates a city from native, recorded or replayed decisions.
    /// The side length must be at least 13, or 26 with a village.
    pub fn generate_with_decisions(
        side: i32,
        options: GenerationOptions,
        decisions: Decisions,
    ) -> io::Result<Self> {
        if side < MIN_TERRAIN_SIDE || (options.village && side < MIN_VILLAGE_SIDE) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "map size too small for generation settings",
            ));
        }
        let mut city = Self::empty_with_decisions(side, decisions)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid map size"))?;
        city.stats.sustainability.old_population = city.people_pool;
        city.coal_veins()?;
        let mut relief = BASE_RELIEF + city.decisions.c_modulo(RELIEF_VARIATION)?;
        let aridity = match options.climate {
            Climate::River => {
                city.decisions.c_modulo(RIVER_ARIDITY_VARIATION)? - RIVER_ARIDITY_OFFSET
            }
            Climate::Desert => {
                city.decisions.c_modulo(CLIMATE_ARIDITY_RANGE)? + CLIMATE_ARIDITY_RANGE
            }
            Climate::Temperate => city.decisions.c_modulo(CLIMATE_ARIDITY_RANGE)?,
            Climate::Swamp => {
                relief /= SWAMP_RELIEF_DIVISOR;
                city.decisions.c_modulo(CLIMATE_ARIDITY_RANGE)? - CLIMATE_ARIDITY_RANGE
            }
        };
        let step = if options.climate == Climate::River {
            city.river_landscape(relief)?
        } else {
            city.fractal_landscape(relief, aridity)?
        };
        city.vegetation(aridity, step.max(1), options.trees)?;
        for tile in &mut city.tiles {
            tile.ore = ORE_RESERVE;
        }
        if options.village {
            city.starting_village()?;
        }
        city.refresh_transport(
            Point { x: 1, y: 1 },
            Point {
                x: side - 2,
                y: side - 2,
            },
        );
        city.update_frontiers(
            Point { x: 1, y: 1 },
            Point {
                x: side - 1,
                y: side - 1,
            },
        );
        Ok(city)
    }
    fn cell(&self, x: i32, y: i32) -> usize {
        (y * self.side + x) as usize
    }
    fn wet_tile(&mut self, x: i32, y: i32) {
        let i = self.cell(x, y);
        let t = &mut self.tiles[i];
        t.ground = Terrain::Water;
        t.frame = 0;
        t.river = true;
        t.groundwater = true;
        t.water_alt = t.altitude;
    }
    fn coal_veins(&mut self) -> io::Result<()> {
        for _ in 0..self.side * self.side / COAL_VEIN_AREA {
            let mut x = self
                .decisions
                .c_modulo(self.side - coal_veins::X_EXCLUDED_TILES)?
                + coal_veins::START_MARGIN;
            let mut y = self
                .decisions
                .c_modulo(self.side - coal_veins::Y_EXCLUDED_TILES)?
                + coal_veins::START_MARGIN;
            let (dx, dy) = loop {
                let dx = self.decisions.c_modulo(3)? - 1;
                let dy = self.decisions.c_modulo(3)? - 1;
                if dx != 0 || dy != 0 {
                    break (dx, dy);
                }
            };
            for _ in 0..coal_veins::VEIN_LENGTH {
                let i = self.cell(x, y);
                self.tiles[i].coal += self.decisions.c_modulo(DEPOSIT_CAPACITY)?;
                x += dx;
                y += dy;
            }
        }
        Ok(())
    }
}

// Equal priorities preserve the heap algorithm's native tie handling. Coordinates
// are payload, never an additional sorting key.
#[derive(Clone, Copy)]
struct Visit {
    index: usize,
    priority: f32,
}
impl PartialEq for Visit {
    fn eq(&self, rhs: &Self) -> bool {
        self.priority == rhs.priority
    }
}
impl Eq for Visit {}
impl PartialOrd for Visit {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
impl Ord for Visit {
    fn cmp(&self, rhs: &Self) -> Ordering {
        rhs.priority.total_cmp(&self.priority)
    }
}
