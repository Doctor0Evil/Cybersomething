//! Global constants for Cybersomething ecosystem

/// Sonoran Desert region constants
pub mod sonoran {
    /// Phoenix, Arizona coordinates
    pub const PHOENIX_LAT: f64 = 33.4484;
    pub const PHOENIX_LON: f64 = -112.0742;

    /// Typical elevation (feet to meters)
    pub const PHOENIX_ELEVATION_M: f64 = 345.0;

    /// Average annual rainfall (mm)
    pub const AVG_RAINFALL_MM: f64 = 203.0;

    /// Growing season length (days)
    pub const GROWING_SEASON_DAYS: u32 = 180;

    /// Peak summer temperature (°C)
    pub const PEAK_TEMP_C: f64 = 45.0;

    /// Winter minimum temperature (°C)
    pub const MIN_TEMP_C: f64 = 5.0;
}

/// Risk index coefficients (calibrated for Sonoran Desert)
pub mod risk_coefficients {
    /// Vegetation density weight (α)
    pub const ALPHA_VEGETATION: f64 = 0.35;

    /// Invasive grass weight (β) — PRIMARY DRIVER
    pub const BETA_GRASS: f64 = 0.45;

    /// Slope steepness weight (γ)
    pub const GAMMA_SLOPE: f64 = 0.20;

    /// Risk thresholds
    pub const RISK_LOW_THRESHOLD: f64 = 0.33;
    pub const RISK_MEDIUM_THRESHOLD: f64 = 0.67;
    pub const RISK_HIGH_THRESHOLD: f64 = 1.0;
}

/// Defensible space zones (meters)
pub mod defensible_zones {
    /// Zero-fuel zone (0-1m)
    pub const ZONE_ZERO_M: u32 = 1;
    /// Fuel reduction zone (1-10m)
    pub const ZONE_REDUCTION_M: u32 = 10;
    /// Extended management (10-30m)
    pub const ZONE_EXTENDED_M: u32 = 30;

    /// Grass height limits (centimeters)
    pub const GRASS_HEIGHT_ZERO_CM: u32 = 0;
    pub const GRASS_HEIGHT_REDUCTION_CM: u32 = 10;
    pub const GRASS_HEIGHT_EXTENDED_CM: u32 = 20;
}

/// Energy constants
pub mod energy {
    /// Drone battery capacity (Wh)
    pub const DRONE_BATTERY_WH: f64 = 500.0;

    /// Nanobot battery capacity (mJ)
    pub const NANOBOT_BATTERY_MJ: f64 = 100.0;

    /// Drone mission baseline energy (Wh)
    pub const DRONE_MISSION_BASELINE_WH: f64 = 50.0;

    /// Nanobot task baseline energy (mJ)
    pub const NANOBOT_TASK_BASELINE_MJ: f64 = 2.0;

    /// Solar panel efficiency (%)
    pub const SOLAR_EFFICIENCY_PERCENT: f64 = 18.0;

    /// RF harvesting efficiency (%)
    pub const RF_EFFICIENCY_PERCENT: f64 = 8.0;
}

/// Ecological thresholds
pub mod ecology {
    /// Minimum tree density for recovery (trees/hectare)
    pub const MIN_TREES_HA_RECOVERY: f64 = 50.0;

    /// Target tree density for mature ecosystem (trees/hectare)
    pub const TARGET_TREES_HA: f64 = 300.0;

    /// Minimum soil organic matter (%)
    pub const MIN_SOIL_ORGANIC_MATTER: f64 = 1.5;

    /// Target soil organic matter (%)
    pub const TARGET_SOIL_ORGANIC_MATTER: f64 = 3.5;

    /// Minimum soil pH
    pub const MIN_PH: f64 = 6.5;

    /// Maximum soil pH
    pub const MAX_PH: f64 = 8.5;

    /// Critical water deficit for irrigation (mm)
    pub const CRITICAL_WATER_DEFICIT_MM: f64 = 50.0;
}

/// Time constants
pub mod time {
    /// Simulation timestep (seconds)
    pub const SIMULATION_STEP_S: u32 = 3600; // 1 hour

    /// Drone mission typical duration (minutes)
    pub const DRONE_MISSION_DURATION_MIN: u32 = 30;

    /// Nanobot task typical duration (minutes)
    pub const NANOBOT_TASK_DURATION_MIN: u32 = 5;

    /// Recovery assessment interval (days)
    pub const RECOVERY_CHECK_INTERVAL_DAYS: u32 = 7;

    /// Full ecosystem recovery timeframe (years)
    pub const FULL_RECOVERY_YEARS: u32 = 10;
}

/// Communication constants
pub mod comms {
    /// API timeout (milliseconds)
    pub const API_TIMEOUT_MS: u64 = 5000;

    /// Telemetry publish interval (seconds)
    pub const TELEMETRY_INTERVAL_S: u32 = 60;

    /// Health check interval (seconds)
    pub const HEALTH_CHECK_INTERVAL_S: u32 = 300;
}
