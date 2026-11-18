//! RustCity Core city simulation.
#![forbid(unsafe_code)]

mod commodity;
pub use commodity::{Commodity, TradePolicy};

mod city;
mod energy;
pub mod stats;
pub use city::{TileView, VehicleFrame};
pub use city::{
    Action, ActionError, City, CityEvent, CityReport, Climate, GenerationOptions, MapSizeError,
    Point, Terrain, VehicleTraffic,
};
mod building_kind;
mod buildings;
pub use building_kind::{BuildDenial, BuildingKind, BuildingMetadata, CommodityRule};

mod power;
mod production;
mod save;
mod workshop;

pub mod decisions;

mod rules;
