//! Constants from the LinCity residence, service and fire modules.
use crate::{BuildingKind, Commodity};

pub(crate) const MARKET_CHECK: u32 = 0x0002_0000;
pub(crate) const FIRE: u32 = 0x0004_0000;
pub(crate) const HEALTH: u32 = 0x0010_0000;
pub(crate) const CRICKET: u32 = 0x0040_0000;
pub(crate) const CHECK_MASK: u32 = MARKET_CHECK | ((FIRE | HEALTH | CRICKET) << 1);
pub(crate) const COVER_MASK: u32 = CHECK_MASK | (CHECK_MASK >> 1);
pub(crate) const FED: u32 = 0x20;
pub(crate) const EMPLOYED: u32 = 0x40;
pub(crate) const HAD_POWER: u32 = 0x0100_0000;
pub(crate) const SCHOOLS_PER_UNIVERSITY: u32 = 4;
pub(crate) const COMPLETED_MONUMENT_FLAGS: u32 = 0x0300;

pub(crate) const COVER_PERIOD: i32 = 3 * super::DAYS_PER_MONTH;
pub(crate) const COVER_DAY: i32 = 75;
pub(crate) const COVER_TOLERANCE: i32 = 5;
pub(crate) const GOODS_PER_WASTE: i32 = 3;
pub(crate) const PARK_CLEANUP_PERIOD: i32 = 2;
pub(crate) const PARK_POLLUTION_FLOOR: i32 = 10;
pub(crate) const WELL_WATER_PER_WET_TILE: i32 = 400;

pub(crate) struct ServiceRule {
    pub labor: i32,
    pub goods: i32,
    pub cost: i32,
    pub tech_cost_factor: i32,
    pub research: i32,
    pub cover: u32,
}
pub(crate) const SCHOOL: ServiceRule = ServiceRule {
    labor: 200,
    goods: 75,
    cost: 2,
    tech_cost_factor: 0,
    research: 2,
    cover: 0,
};
pub(crate) const UNIVERSITY: ServiceRule = ServiceRule {
    labor: 250,
    goods: 750,
    cost: 23,
    tech_cost_factor: 0,
    research: 4,
    cover: 0,
};
pub(crate) const FIRE_STATION: ServiceRule = ServiceRule {
    labor: 6,
    goods: 3,
    cost: 1,
    tech_cost_factor: 6,
    research: 0,
    cover: FIRE << 1,
};
pub(crate) const HEALTH_CENTRE: ServiceRule = ServiceRule {
    labor: 6,
    goods: 40,
    cost: 2,
    tech_cost_factor: 9,
    research: 0,
    cover: HEALTH << 1,
};
pub(crate) const SPORTS: ServiceRule = ServiceRule {
    labor: 8,
    goods: 3,
    cost: 1,
    tech_cost_factor: 0,
    research: 0,
    cover: CRICKET << 1,
};
pub(crate) const SPORTS_FRAMES: i32 = 7;
pub(crate) const SPORTS_FRAME_MS: u64 = 750;
pub(crate) const STATION_FRAMES: i32 = 10;
pub(crate) const STATION_FRAME_MS: u64 = 250;
pub(crate) const STATION_HOLD_FRAME: i32 = 6;
pub(crate) const STATION_HOLD_MS: u64 = 11 * STATION_FRAME_MS;
pub(crate) const SWING_FRAME_MS: u64 = 80;
pub(crate) const SWING_FRAMES: i32 = 10;
pub(crate) const SWING_PAUSE_MS: i64 = 9500;
pub(crate) const SWING_MS_PER_WORKDAY: i32 = 100;
pub(crate) const SWING_MIN_WORKDAYS: i32 = 20;

pub(crate) const MONUMENT_LABOR_PER_BATCH: i32 = 100;
pub(crate) const MONUMENT_TOTAL_LABOR: i32 = 350_000;
pub(crate) const MONUMENT_RESEARCH_CEILING: i32 = 400_000;
pub(crate) const MONUMENT_RESEARCH_INTERVAL: i32 = 3;
pub(crate) const MONUMENT_RESEARCH_PHASE: i32 = 1;
pub(crate) const MONUMENT_TECH_PER_DELAY: i32 = 10_000;
pub(crate) const MONUMENT_DELAY_DISCOUNT: i32 = 2;
pub(crate) const MONUMENT_PERCENT_PER_FRAME: i32 = 20;

pub(crate) const SHANTY_BATCHES: [(Commodity, i32); 6] = [
    (Commodity::Food, 50),
    (Commodity::Labor, 5),
    (Commodity::Goods, 50),
    (Commodity::Coal, 1),
    (Commodity::Ore, 10),
    (Commodity::Steel, 1),
];
pub(crate) const SHANTY_DAILY_WASTE: i32 = 2;
pub(crate) const SHANTY_TAX_MULTIPLIER: i32 = 2;
pub(crate) const UNCONTROLLED_WASTE_FIRE: f64 = 0.001;
pub(crate) const SHANTY_POOL_MIN: i32 = 150;
pub(crate) const SHANTY_CAPACITY: i32 = 50;
pub(crate) const SHANTY_MONTHLY_DEATHS: i32 = 5;
pub(crate) const COMMUNE_CAPACITY: i32 = 5;
pub(crate) const SHANTY_CHANGE_DIVISOR: i32 = 10;
pub(crate) const SHANTY_NEW_CLUSTER_ODDS: i32 = 8;
pub(crate) const SHANTY_ADJUST_DAY: i32 = 15;

pub(crate) const FIRE_BURNING_DAYS: i32 = 5 * super::DAYS_PER_YEAR;
pub(crate) const FIRE_SMOKING_DAYS: i32 = 10 * super::DAYS_PER_YEAR;
pub(crate) const FIRE_INITIAL_BURN_SPAN: i32 = FIRE_BURNING_DAYS / 5;
pub(crate) const FIRE_INITIAL_SMOKE_SPAN: i32 = FIRE_SMOKING_DAYS / 6;
pub(crate) const FIRE_SPREAD_DAYS: i32 = super::DAYS_PER_YEAR / 8;
pub(crate) const FIRE_SPREAD_ODDS: i32 = 80;
pub(crate) const FIRE_PROTECTION_ACCELERATION: i32 = 4;
pub(crate) const FIRE_CASUALTY_DIVISOR: i32 = 2;
pub(crate) const SHANTY_DEMOLITION_FIRE_AGE: i32 = FIRE_BURNING_DAYS - 25;
pub(crate) const FIRE_FRAMES: i32 = 5;
pub(crate) const FINAL_SMOKE_FRAME: i32 = 4;
pub(crate) const FIRE_MIN_FRAME_MS: i32 = 25;
pub(crate) const FIRE_FRAME_VARIATION_MS: i32 = 350;
pub(crate) const ANIMATION_QUANTUM_MS: i32 = 100;
pub(crate) const WASTE_FIRE_VISIBLE_MS: u64 = 1050;

pub(crate) mod housing {
    pub const BASE_APPEAL: i32 = 30;
    pub const BASE_DISCONTENT: i32 = 35;
    pub const HEALTH_APPEAL: i32 = 15;
    pub const FIRE_APPEAL: i32 = 15;
    pub const SPORTS_APPEAL: i32 = 20;
    pub const NO_FIRE_PENALTY: i32 = 5;
    pub const FED_APPEAL: i32 = 10;
    pub const STARVATION_PENALTY: i32 = 250;
    pub const POWER_APPEAL: i32 = 10;
    pub const POWER_SHORTAGE_PENALTY: i32 = 15;
    pub const LOST_POWER_PENALTY: i32 = 50;
    pub const EMPLOYMENT_APPEAL: i32 = 20;
    pub const GOODS_APPEAL: i32 = 10;
    pub const APPLIANCE_POWER_APPEAL: i32 = 5;
    pub const EXTRA_GOODS_APPEAL: i32 = 5;
    pub const APPLIANCE_SHORTAGE_PENALTY: i32 = 5;
    pub const UNEMPLOYMENT_PENALTY: i32 = 70;
    pub const POLLUTION_DEATH_PENALTY: i32 = 100;
    pub const BIRTH_APPEAL: i32 = 50;
    pub const WORKING_PERCENT: i32 = 58;
    pub const JOB_SWING: i32 = 2;
    pub const HEALTH_WORKERS: i32 = 8;
    pub const HEALTH_SWING: i32 = 2;
    pub const SPORTS_WORKERS: i32 = 3;
    pub const SPORTS_SWING: i32 = 1;
    pub const STARVATION_ODDS: i32 = 20;
    pub const STARVATION_DEATH_MODIFIER: i32 = 100;
    pub const HEALTH_BIRTH_MODIFIER: i32 = 300;
    pub const APPLIANCE_BIRTH_MODIFIER: i32 = 10;
    pub const POWER_OVERHEAD: i32 = 30;
    pub const POWER_PER_RESIDENT: i32 = 3;
    pub const PEOPLE_PER_GOOD: i32 = 4;
    pub const PEOPLE_PER_WASTE: i32 = 12;
    pub const PEOPLE_PER_APPLIANCE_POWER: i32 = 2;
    pub const PEOPLE_PER_RATE_MODIFIER: i32 = 4;
    pub const BASE_BIRTH_INTERVAL: i32 = 100;
    pub const BASE_DEATH_INTERVAL: i32 = 816;
    pub const INACTIVE_INTERVAL: i32 = 120_000;
    pub const POLLUTION_UNIT: i32 = 16;
    pub const FATAL_POLLUTION_OFFSET: i32 = 5;
    pub const FATAL_POLLUTION_MAX_PERCENT: i32 = 95;
    pub const POLLUTION_DEATH_FACTOR: i32 = 3;
    pub const HEALTH_SURVIVAL_FACTOR: i32 = 4;
    pub const POLLUTION_PER_DISCONTENT: i32 = 20;
    pub const UNHOUSED_PER_APPEAL: i32 = 27;
    pub const MIGRATION_ODDS: i32 = 20;
    pub const MIN_OCCUPANTS: i32 = 10;
}

/// Death density truncates before multiplication in medium-high housing.
pub(crate) struct HousingRule {
    pub occupancy_divisor: i32,
    pub density_factor: i32,
    pub density_divisor: i32,
    pub birth_delay: i32,
    pub appeal: i32,
    pub capacity: i32,
}
pub(crate) fn housing_rule(kind: BuildingKind) -> HousingRule {
    let (occupancy_divisor, density_factor, density_divisor, birth_delay, appeal, capacity) =
        match kind {
            BuildingKind::ResidenceLowLow => (1, 7 * 50, 24, 1200, 0, 50),
            BuildingKind::ResidenceMediumLow => (1, 3 * 5, 3, 0, 0, 100),
            BuildingKind::ResidenceHighLow => (1, 6, 3, 125, 40, 200),
            BuildingKind::ResidenceLowHigh => (1, 3 * 7, 3, 1800, 0, 100),
            BuildingKind::ResidenceMediumHigh => (2, 4, 3, 50, 0, 200),
            BuildingKind::ResidenceHighHigh => (1, 4, 3, 0, 100, 400),
            _ => unreachable!("residence rules require a residence"),
        };
    HousingRule {
        occupancy_divisor,
        density_factor,
        density_divisor,
        birth_delay,
        appeal,
        capacity,
    }
}

pub(crate) const WELL_MAX_DRINKABLE_POLLUTION: i32 = 3000;
