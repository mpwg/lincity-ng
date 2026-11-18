//! Resource recipes, work cycles and animation timing for handicraft buildings.
use crate::Commodity::{self, Coal, Labor, Ore, Steel};

pub(crate) const LABOR_PER_BATCH: i32 = 35;
pub(crate) const SMITHY_OUTPUT: i32 = 50;
pub(crate) const SMITHY_INPUTS: [(Commodity, i32); 3] =
    [(Coal, 1), (Steel, 1), (Labor, LABOR_PER_BATCH)];
pub(crate) const POTTERY_OUTPUT: i32 = 35;
pub(crate) const POTTERY_INPUTS: [(Commodity, i32); 3] =
    [(Coal, 2), (Ore, 11), (Labor, LABOR_PER_BATCH)];
pub(crate) const MILL_OUTPUT: i32 = 75;
pub(crate) const MILL_FOOD: i32 = 37;
pub(crate) const MILL_POWER_PER_COAL: i32 = 60;
pub(crate) const SMITHY_BATCHES_PER_EMISSION: i32 = 100;
pub(crate) const MILL_BATCHES_PER_EMISSION: i32 = 7;
pub(crate) const POTTERY_DAYS_PER_EMISSION: i32 = 10;
pub(crate) const POTTERY_RESTART_DAYS: u8 = 25;
/// Frame interval in milliseconds and number of frames in the sprite.
pub(crate) const SMITHY_ANIMATION: (u64, u16) = (150, 7);
pub(crate) const MILL_ANIMATION: (u64, u16) = (250, 7);
pub(crate) const POTTERY_ANIMATION: (u64, u16) = (250, 11);
