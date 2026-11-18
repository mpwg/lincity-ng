//! Shared simulation constants. Save identifiers and per-building metadata
//! live in the construction catalog; numerical expectations stay in fixtures.
pub(crate) const DAYS_PER_MONTH: i32 = 100;
pub(crate) const MONTHS_PER_YEAR: i32 = 12;
pub(crate) const DAYS_PER_YEAR: i32 = DAYS_PER_MONTH * MONTHS_PER_YEAR;
pub(crate) const MAX_TECH_LEVEL: i32 = 1_000_000;
pub(crate) const CREDIT_LIMIT: i32 = -2_000_000_000;
pub(crate) const YEAR_END_CASH_CEILING: i32 = 2_000_000_000;
pub(crate) const FLOOD_COST: i32 = 1_000_000;
pub(crate) const DRAIN_COST: i32 = 1_000_000;
pub(crate) const CLEAR_LAND_COST: i32 = 1;

pub(crate) mod civic;
pub(crate) mod commune;
pub(crate) mod economy;
pub(crate) mod environment;
pub(crate) mod farm;
pub(crate) mod generation;
pub(crate) mod industry;
pub(crate) mod market;
pub(crate) mod power;
pub(crate) mod rocket;
pub(crate) mod statistics;
pub(crate) mod transport;
pub(crate) mod workshop;
