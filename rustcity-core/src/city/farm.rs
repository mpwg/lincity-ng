//! Organic farm production and its randomized crop rotation.
use super::*;
use crate::{Commodity, power::ProductionCounters};

use crate::rules::{DAYS_PER_MONTH, DAYS_PER_YEAR, MAX_TECH_LEVEL};

use crate::rules::farm::*;

pub(super) struct Farm {
    pub(super) max_food: i32,
    rotation: i32,
    stagger: i32,
    pub(super) groundwater: i32,
    utilization: i32,
    pub(super) last_utilization: i32,
    pub(super) frame: u8,
}

impl Farm {
    pub(super) fn new(decisions: &mut Decisions) -> io::Result<Self> {
        Ok(Self {
            max_food: 0,
            rotation: decisions.c_modulo(CROP_ROTATIONS)? + 1,
            stagger: decisions.c_modulo(DAYS_PER_MONTH)?,
            groundwater: 0,
            utilization: 0,
            last_utilization: 0,
            frame: 0,
        })
    }

    pub(super) fn update(
        &mut self,
        day: i32,
        tech: i32,
        stock: &mut [i32; 10],
        counters: &mut ProductionCounters,
    ) {
        use Commodity::*;
        let before = *stock;
        let mut labor = stock[Labor as usize].min(LABOR_PER_BATCH);
        let mut power = 0;
        let mut water = 0;
        let potential = if stock[LowVoltage as usize] >= POWER_PER_BATCH {
            power = POWER_PER_BATCH;
            if stock[Waste as usize] >= 3 * COMPOST_PER_BATCH {
                stock[Waste as usize] -= COMPOST_PER_BATCH;
            }
            water = (stock[Water as usize] / WATER_PER_TILE).min(FIELD_TILES - self.groundwater);
            let output = FOOD_PER_BATCH
                + (i64::from(tech) * i64::from(FOOD_PER_BATCH) / i64::from(MAX_TECH_LEVEL)) as i32;
            output * (self.groundwater + water)
        } else {
            FOOD_PER_BATCH * self.groundwater / UNPOWERED_OUTPUT_DIVISOR
        };
        let mut food = (potential * labor / (FIELD_TILES * LABOR_PER_BATCH)).max(MIN_FOOD_BATCH);
        let max_food = (potential / FIELD_TILES).max(MIN_FOOD_BATCH);
        self.max_food = max_food;
        let room = BuildingKind::OrganicFarm.metadata().rule(Food).max_load - stock[Food as usize];
        if food > room {
            labor = labor * room / food;
            power = power * room / food;
            if room * FIELD_TILES < self.groundwater * food {
                water = 0;
            }
            food = room;
        }
        if food >= MIN_FOOD_BATCH {
            stock[Labor as usize] -= labor;
            stock[Food as usize] += food;
            stock[LowVoltage as usize] -= power;
            stock[Water as usize] -= water * WATER_PER_TILE;
            self.utilization += 100 * food / max_food;
        }
        for i in 0..stock.len() {
            counters.current[i] += stock[i] - before[i];
        }
        if day % DAYS_PER_MONTH == DAYS_PER_MONTH - 1 {
            counters.close_month();
            self.last_utilization = std::mem::take(&mut self.utilization);
        }
    }

    pub(super) fn animate(&mut self, day: i32, decisions: &mut Decisions) -> io::Result<()> {
        let phase = (i64::from(day) + i64::from(self.rotation * DAYS_PER_YEAR + self.stagger))
            % i64::from(CROP_ROTATIONS * DAYS_PER_YEAR);
        if phase % i64::from(3 * DAYS_PER_MONTH) == 0 {
            self.frame = if self.last_utilization > MIN_ANIMATED_UTILIZATION {
                if phase % i64::from(DAYS_PER_YEAR) == 0 {
                    self.stagger = decisions.c_modulo(DAYS_PER_MONTH)?;
                }
                (1 + phase / i64::from(DAYS_PER_YEAR)) as u8
            } else {
                0
            };
        }
        Ok(())
    }
}
