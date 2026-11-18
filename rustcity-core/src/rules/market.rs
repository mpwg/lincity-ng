//! Market staffing, inventory and waste disposal.
pub(crate) const STAFF_REVIEW_PERIOD: i32 = 25;
pub(crate) const STAFF_REVIEW_DAY: i32 = 17;
pub(crate) const MIN_LABOR: i32 = 1;
/// Minimum fill percentage and corresponding labor requirement, highest first.
pub(crate) const STAFF_LEVELS: [(i32, i32); 3] = [(50, 28), (20, 12), (10, 5)];
pub(crate) const WASTE_REST_PERIOD: i32 = 50;
pub(crate) const WASTE_THRESHOLD_PERCENT: i32 = 85;
pub(crate) const WASTE_BURN_NUMERATOR: i32 = 7;
pub(crate) const WASTE_BURN_DENOMINATOR: i32 = 10;
pub(crate) const POLLUTION_DIVISOR: i32 = 20;
pub(crate) const FIRE_VISIBLE_MS: u64 = 2050;
