//! RustCity Core city simulation.
#![forbid(unsafe_code)]

mod commodity;
pub use commodity::Commodity;

mod city;
mod energy;
mod stats;
pub use city::{Action, ActionError, City, MapSizeError, Point, Terrain};
