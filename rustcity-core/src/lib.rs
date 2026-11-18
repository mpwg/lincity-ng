//! RustCity Core city simulation.
#![forbid(unsafe_code)]

mod commodity;
pub use commodity::Commodity;

mod city;
mod energy;
mod stats;
pub use city::VehicleFrame;
pub use city::{
    Action, ActionError, City, CityEvent, CityReport, MapSizeError, Point, Terrain, VehicleTraffic,
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
