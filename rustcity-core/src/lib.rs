//! RustCity Core city simulation.
#![forbid(unsafe_code)]

mod commodity;
pub use commodity::Commodity;

mod city;
mod energy;
mod stats;
pub use city::{Action, ActionError, City, MapSizeError, Point, Terrain};
mod building_kind;
mod buildings;
pub use building_kind::{BuildDenial, BuildingKind, BuildingMetadata, CommodityRule};
