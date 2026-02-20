//! Error types for Cybersomething

use thiserror::Error;

/// Primary error type
#[derive(Debug, Error)]
pub enum CybersomethingError {
    #[error("Invalid coordinate: latitude {lat}, longitude {lon}")]
    InvalidCoordinate { lat: f64, lon: f64 },

    #[error("Zone not found: {zone_id}")]
    ZoneNotFound { zone_id: u32 },

    #[error("Device not found: {device_id}")]
    DeviceNotFound { device_id: u64 },

    #[error("Insufficient energy: needed {needed_wh:.1} Wh, available {available_wh:.1} Wh")]
    InsufficientEnergy { needed_wh: f64, available_wh: f64 },

    #[error("Mission planning failed: {reason}")]
    MissionPlanningFailed { reason: String },

    #[error("Route calculation failed: no path found")]
    NoRoutePath,

    #[error("Hardware not available: {hardware_type}")]
    HardwareUnavailable { hardware_type: String },

    #[error("Simulation error: {reason}")]
    SimulationError { reason: String },

    #[error("Data validation error: {reason}")]
    DataValidationError { reason: String },

    #[error("ALN compliance violation: {reason}")]
    ALNComplianceViolation { reason: String },

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error")]
    Unknown,
}

/// Result type for Cybersomething operations
pub type Result<T> = std::result::Result<T, CybersomethingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CybersomethingError::ZoneNotFound { zone_id: 42 };
        assert_eq!(format!("{}", err), "Zone not found: 42");
    }

    #[test]
    fn test_insufficient_energy_error() {
        let err = CybersomethingError::InsufficientEnergy {
            needed_wh: 100.0,
            available_wh: 50.0,
        };
        assert!(format!("{}", err).contains("100.0"));
    }
}
