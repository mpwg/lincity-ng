//! Transport and commuter rules from transport.hpp, Vehicles.hpp and all_buildings.hpp.
/// Divisor limiting each transport transfer relative to link capacity.
pub const RATE: i32 = 8;
/// Fixed-point scale for stock and traffic ratios.
pub const QUANTA: i32 = 1000;
pub(crate) const SMOOTHING: i32 = 10;
pub(crate) const DEFAULT_COST: i32 = 14;
pub(crate) const RAIL_COST_MULTIPLIER: i32 = 3;
pub(crate) const ROAD_POLLUTION_DAYS: i32 = 20;
pub(crate) const RAIL_POLLUTION_DAYS: i32 = 30;
pub(crate) const ROAD_GOODS_MASK: i32 = 0x1f;
pub(crate) const RAIL_GOODS_MASK: i32 = 0xf;
pub(crate) const RAIL_STEEL_MASK: i32 = 0x7f;
pub(crate) const VOLTAGE_LOSS: i32 = 3;
pub(crate) const WASTE_FIRE_VISIBLE_MS: u64 = 350;
pub(crate) const BURN_WASTE: i32 = 100;
pub(crate) const BURN_POLLUTION: i32 = BURN_WASTE / 2;
pub(crate) const UNCONTROLLED_FIRE: f64 = 0.001;
pub(crate) const COMMUTER_TRAFFIC_RATE: i32 = 1024;
pub(crate) const CAR_SPEED: u64 = 1500;
pub(crate) const CAR_RANGE: i32 = 100;
pub(crate) const ROAD_BRIDGE_HEIGHT: i32 = 44;
pub(crate) const TRACK_BRIDGE_HEIGHT: i32 = 22;
pub(crate) const HIDDEN_CAR_INTERVAL: u64 = 50;
pub(crate) const TRAFFIC_REPORT_FACTOR: f64 = 107.77;

pub(crate) const NORTH_SOUTH_CABLE: u8 = 1;
pub(crate) const EAST_WEST_CABLE: u8 = 2;
