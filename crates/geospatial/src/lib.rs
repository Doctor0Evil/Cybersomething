//! Cybersomething Geospatial Module
//!
//! Spatial indexing, raster operations, and geographic data handling for
//! efficient large-scale ecosystem monitoring and recovery planning.
//!
//! # Modules
//!
//! - `grid` — Regular spatial grids for zone management
//! - `raster` — Raster datasets (UAV, satellite imagery)
//! - `vector` — Vector geometries (polygons, points, lines)
//! - `projection` — Coordinate system transformations

pub mod grid;
pub mod raster;
pub mod vector;
pub mod projection;

pub use grid::*;
pub use raster::*;
pub use vector::*;
pub use projection::*;
