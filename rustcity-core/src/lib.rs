//! A LinCity simulation owned and driven by the host application.
//!
//! [`City`] loads an existing city or generates a new map. Apply [`Action`]s,
//! advance the simulation with [`City::tick`], and drain [`CityEvent`]s with
//! [`City::pop_event`]. Advance animation separately using a millisecond clock.
//! Attach [`decisions::Decisions`] during creation to record or replay a run.
//!
//! ```
//! use rustcity_core::City;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut city = City::empty(32)?;
//! city.animate(1_000)?;
//! city.tick()?;
//! let mut bytes = Vec::new();
//! city.save(&mut bytes)?;
//! let resumed = City::load(bytes.as_slice())?;
//! assert_eq!(resumed.report().day, city.report().day);
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]

mod commodity;
pub use commodity::{Commodity, TradePolicy};

mod city;
mod energy;
pub mod stats;
pub use city::{
    Action, ActionError, ActionQuote, City, CityEvent, CityReport, Climate, GenerationOptions,
    MapSizeError, Point, Terrain, VehicleTraffic,
};
pub use city::{BuildingState, Landform, RocketStage, TileView, VehicleFrame};
pub use city::{BuildingVisual, ReportMode, ReportRow, SpriteLayer};
mod building_kind;
mod buildings;
pub use building_kind::{BuildDenial, BuildingKind, BuildingMetadata, CommodityRule};

mod power;
mod production;
mod save;
mod workshop;

pub mod decisions;

mod rules;
pub use rules::environment::{
    FIRE_YEARS, MINING_YEARS, MONEY_YEARS, POPULATION_YEARS, TECHNOLOGY_YEARS, TRADE_YEARS,
};
pub use rules::industry::coal_mine::DEPOSIT_CAPACITY as COAL_DEPOSIT_CAPACITY;
pub use rules::transport::{QUANTA as TRANSPORT_QUANTA, RATE as TRANSPORT_RATE};
pub use rules::{DAYS_PER_MONTH, DAYS_PER_YEAR, FLOOD_COST, MAX_TECH_LEVEL};
