//! Mining, industry and waste rules from lincity/all_buildings.hpp and modules.
pub(crate) const LOAD_COAL: i32 = 18;
pub(crate) const LOAD_ORE: i32 = 9;
pub(crate) const LOAD_STEEL: i32 = 15;
pub(crate) const ORE_RESERVE: i32 = 1000;

pub(crate) mod coal_mine {
    pub const RANGE: i32 = 6;
    pub const MINIMUM_DEPOSITS: i32 = 20;
    pub const DEPOSIT_CAPACITY: i32 = 10_000;
    pub const BATCH: i32 = 1000;
    pub const LABOR: i32 = 900 + super::LOAD_COAL;
    pub const STOCK_CAPACITY: i32 = 20 * BATCH;
    pub const TARGET: i32 = STOCK_CAPACITY * 80 / 100;
    pub const POLLUTION: i32 = 3;
}
pub(crate) mod ore_mine {
    pub const SIZE: i32 = 4;
    pub const BATCH: i32 = 5000;
    pub const LABOR: i32 = 200 + super::LOAD_ORE;
    pub const CAPACITY: i32 = 20 * BATCH;
    pub const TARGET_PERCENT: i32 = 80;
    pub const SUSTAINABLE_DEPOSITS: i32 = SIZE * SIZE * super::ORE_RESERVE;
    pub const FRAME_SPEED: i32 = 200;
}
pub(crate) mod light {
    pub const ORE: i32 = 125;
    pub const STEEL: i32 = 12;
    pub const BASE_LABOR: i32 = 30 + 1 + super::LOAD_ORE;
    pub const STEEL_LABOR: i32 = 4 + super::LOAD_STEEL;
    pub const GOODS: i32 = 225;
    pub const STEEL_MULTIPLIER: i32 = 2;
    pub const POWER_MULTIPLIER: i32 = 4;
    pub const MAX_OUTPUT: i32 = GOODS * STEEL_MULTIPLIER * POWER_MULTIPLIER;
    pub const GOODS_CAPACITY: i32 = 20 * MAX_OUTPUT;
    pub const WASTE_CAPACITY: i32 = GOODS_CAPACITY / 20;
    pub const POWER_PER_GOOD: i32 = 10;
    pub const POLLUTION_PER_GOOD: f64 = 0.05;
}
pub(crate) mod heavy {
    pub const BATCH_ORE: i32 = 612;
    pub const ORE_PER_STEEL: i32 = 17;
    pub const OUTPUT: i32 = BATCH_ORE / ORE_PER_STEEL;
    pub const STEEL_CAPACITY: i32 = 20 * BATCH_ORE / ORE_PER_STEEL;
    pub const WASTE_CAPACITY: i32 = STEEL_CAPACITY / 4;
    pub const COAL_PER_STEEL: i32 = 2;
    pub const POWER_PER_STEEL: i32 = 200;
    pub const LOW_POWER_DIVISOR: i32 = 5;
    pub const LABOR: i32 = BATCH_ORE / 140 + super::LOAD_ORE + super::LOAD_STEEL;
    pub const POLLUTION_PER_STEEL: f64 = 0.25;
}
pub(crate) mod recycle {
    pub const WASTE: i32 = 500;
    pub const LABOR: i32 = WASTE / 50 + super::LOAD_ORE + super::LOAD_STEEL;
    pub const POWER: i32 = WASTE / 2;
    pub const RUNNING_COST: i32 = 3;
    pub const MAX_ORE: i32 = 16 * WASTE;
    pub const MAX_STEEL: i32 = MAX_ORE / 50;
    pub const STOCKED_WASTE: i32 = 20 * WASTE;
    pub const BURN: i32 = STOCKED_WASTE / 200;
}
pub(crate) mod tip {
    pub const CAPACITY: i32 = 10_000_000;
    pub const BATCH: i32 = 200;
    pub const INBOX: i32 = 20 * BATCH;
    pub const TARGET_PERCENT: i32 = 20;
    pub const DEGRADE_DAYS: i32 = 200 * crate::rules::DAYS_PER_YEAR;
}
