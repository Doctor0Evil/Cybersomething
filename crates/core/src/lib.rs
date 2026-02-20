//! Cybersomething Core Library
//! 
//! Provides foundational domain models, mathematical computations, and utilities
//! for ecological recovery and deforestation prevention systems.
//!
//! # Modules
//!
//! - `models` — Geospatial, ecological, and hardware domain types
//! - `math` — Risk indexing, routing, hydrology, and energy calculations
//! - `utils` — Constants, helpers, error handling

pub mod models;
pub mod math;
pub mod utils;

pub use models::*;
pub use math::*;
pub use utils::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing (call once at app startup)
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}
