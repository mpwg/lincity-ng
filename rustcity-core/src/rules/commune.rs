//! Forest production, groundwater thresholds and abandonment.
pub(crate) const COAL_BATCH: i32 = 3;
pub(crate) const ORE_BATCH: i32 = 6;
pub(crate) const ORE_FROM_WASTE: i32 = 1;
pub(crate) const STEEL_BATCH: i32 = 2;
pub(crate) const WASTE_BATCH: i32 = 20;
pub(crate) const WASTE_RESERVE_BATCHES: i32 = 3;
pub(crate) const WATER_PER_DRY_TILE: i32 = 5;
pub(crate) const LOW_FERTILITY_DIVISOR: i32 = 3;
pub(crate) const MEDIUM_FERTILITY_DIVISOR: i32 = 2;
pub(crate) const FERTILITY_TIERS: i32 = 3;
pub(crate) const POLLUTION_CLEANUP_INTERVAL: i32 = 10;
pub(crate) const STEEL_INTERVAL: i32 = 2 * POLLUTION_CLEANUP_INTERVAL;
pub(crate) const ABANDONMENT_MONTHS: i32 = 10 * super::MONTHS_PER_YEAR;
pub(crate) const FRAME_INTERVAL_MS: i32 = 750;
pub(crate) const FRAME_JITTER_MS: i32 = 50;
pub(crate) const FRAMES_PER_ACTIVITY: u8 = 5;

pub(crate) const OUTPUT_PER_BUSY_PERCENT: f64 = 3.05;
