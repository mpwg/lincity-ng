//! Rocket-pad rules from rocket_pad.hpp and the launch result calculation.
pub(crate) const STEPS: i32 = 5_000;
pub(crate) const FRAME_STEPS: [i32; 4] = [STEPS / 4, STEPS * 60 / 100, STEPS * 90 / 100, STEPS];
pub(crate) const LAUNCH_FRAME: u8 = 6;
pub(crate) const FINISHED_FRAME: u8 = 7;
pub(crate) const FRAME_INTERVAL_MS: u64 = 750;
pub(crate) const RUNNING_COST: i32 = 200;
pub(crate) const LABOR: i32 = 200;
pub(crate) const GOODS: i32 = 10_000;
pub(crate) const STEEL: i32 = 240;
pub(crate) const WASTE: i32 = GOODS / 3;
pub(crate) const INITIAL_SUCCESS_FACTOR: f64 = 0.25;
pub(crate) const SUCCESS_GAIN_PER_LAUNCH: f64 = 0.15;
pub(crate) const QUALIFYING_LAUNCHES: i32 = 5;
pub(crate) const PASSENGER_CAPACITY: i32 = 1_000;
pub(crate) const CRASH_DEVIATION: f64 = 20.0;
pub(crate) const FIRE_DEVIATION: f64 = 10.0;
pub(crate) const CRASH_FIRE_ATTEMPTS: usize = 20;
