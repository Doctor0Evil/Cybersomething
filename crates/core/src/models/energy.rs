//! Energy models (hardware-only power budgets, completely decoupled from biology)

use serde::{Deserialize, Serialize};

/// Energy source type (hardware-only, non-biological)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergySource {
    Battery,           // Lithium or solid-state
    Solar,             // Photovoltaic panels
    RadioFrequency,    // RF harvesting (for nanobots)
    Wind,              // Miniature turbines
    Kinetic,           // Motion-based charging
}

impl EnergySource {
    /// Energy generation rate (Watts) under nominal conditions
    pub fn nominal_power_w(&self) -> f64 {
        match self {
            Self::Battery => 0.0,       // Passive storage
            Self::Solar => 100.0,       // 100W @ 1000 W/m² irradiance
            Self::RadioFrequency => 0.5, // Weak RF field
            Self::Wind => 50.0,         // 10 m/s wind
            Self::Kinetic => 5.0,       // Low motion
        }
    }

    /// Peak capacity (Joules or Wh)
    pub fn peak_capacity(&self, device_type: &str) -> f64 {
        match self {
            Self::Battery => match device_type {
                "drone" => 1800.0,   // 500 Wh
                "nanobot" => 0.36,   // 100 mJ
                _ => 100.0,
            },
            Self::Solar => f64::INFINITY, // Unlimited if sunny
            Self::RadioFrequency => f64::INFINITY,
            Self::Wind => f64::INFINITY,
            Self::Kinetic => f64::INFINITY,
        }
    }

    /// Availability probability (0.0-1.0) in Sonoran Desert
    pub fn availability_sonoran(&self) -> f64 {
        match self {
            Self::Battery => 1.0,  // Always available
            Self::Solar => 0.85,   // 350 sunny days/year in Phoenix
            Self::RadioFrequency => 0.9, // Relay networks
            Self::Wind => 0.3,     // Low wind in desert floor
            Self::Kinetic => 0.6,  // Moderate motion
        }
    }
}

/// Power budget for a device (daily cycle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerBudget {
    pub device_id: u64,
    pub device_type: String,
    pub total_energy_budget_wh: f64,    // Daily allowance
    pub energy_consumed_wh: f64,        // Today's consumption
    pub peak_power_w: f64,              // Maximum draw
    pub energy_sources: Vec<EnergySource>,
}

impl PowerBudget {
    pub fn new(device_id: u64, device_type: String, budget_wh: f64) -> Self {
        Self {
            device_id,
            device_type,
            total_energy_budget_wh: budget_wh,
            energy_consumed_wh: 0.0,
            peak_power_w: 100.0, // Typical max
            energy_sources: vec![],
        }
    }

    /// Remaining energy in Wh
    pub fn remaining_wh(&self) -> f64 {
        (self.total_energy_budget_wh - self.energy_consumed_wh).max(0.0)
    }

    /// State of charge (0.0-1.0)
    pub fn state_of_charge(&self) -> f64 {
        self.remaining_wh() / self.total_energy_budget_wh
    }

    /// Can draw this much power without exceeding budget?
    pub fn can_draw(&self, power_w: f64, duration_minutes: f64) -> bool {
        let energy_needed = power_w * duration_minutes / 60.0;
        energy_needed <= self.remaining_wh()
    }

    /// Consume energy
    pub fn draw_energy(&mut self, energy_wh: f64) -> bool {
        if energy_wh <= self.remaining_wh() {
            self.energy_consumed_wh += energy_wh;
            true
        } else {
            false
        }
    }

    /// Recharge from energy source
    pub fn recharge(&mut self, source: EnergySource, energy_wh: f64) {
        let old = self.energy_consumed_wh;
        self.energy_consumed_wh = (self.energy_consumed_wh - energy_wh).max(0.0);
        
        if !self.energy_sources.contains(&source) {
            self.energy_sources.push(source);
        }
    }

    /// Daily recovery percentage based on sources
    pub fn daily_recovery_percent(&self) -> f64 {
        if self.energy_sources.is_empty() {
            return 0.0;
        }

        let avg_recovery = self.energy_sources
            .iter()
            .map(|src| src.nominal_power_w() * 24.0 / self.total_energy_budget_wh * src.availability_sonoran())
            .sum::<f64>()
            / self.energy_sources.len() as f64;

        (avg_recovery * 100.0).min(100.0)
    }
}

/// Energy expenditure log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyEvent {
    pub timestamp_s: u64,
    pub device_id: u64,
    pub event_type: EnergyEventType,
    pub power_w: f64,
    pub duration_s: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyEventType {
    Mission,        // Operational task
    Idle,          // Standby/sleep
    Charging,      // Recharge cycle
    Thermal,       // Temperature regulation
    Communication, // Radio/telemetry
    Processing,    // Computation
}

impl EnergyEvent {
    /// Energy consumed in this event (Wh)
    pub fn energy_wh(&self) -> f64 {
        self.power_w * self.duration_s as f64 / 3600.0
    }
}

/// System-wide power distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerGrid {
    pub grid_id: u32,
    pub solar_capacity_w: f64,
    pub wind_capacity_w: f64,
    pub battery_capacity_wh: f64,
    pub battery_soc_percent: f64,     // State of charge
    pub connected_devices: Vec<u64>,  // Device IDs
    pub efficiency_percent: f64,      // Distribution loss
}

impl PowerGrid {
    pub fn new(grid_id: u32) -> Self {
        Self {
            grid_id,
            solar_capacity_w: 500.0,
            wind_capacity_w: 100.0,
            battery_capacity_wh: 10000.0,
            battery_soc_percent: 80.0,
            connected_devices: Vec::new(),
            efficiency_percent: 95.0,
        }
    }

    /// Total available power in Watts (accounting for Sonoran Desert solar)
    pub fn available_power_w(&self) -> f64 {
        let solar = self.solar_capacity_w * 0.85; // Sonoran Desert availability
        let wind = self.wind_capacity_w * 0.3;   // Low desert wind
        let battery_discharge = (self.battery_capacity_wh * self.battery_soc_percent / 100.0) / 24.0;

        (solar + wind + battery_discharge) * self.efficiency_percent / 100.0
    }

    /// Can grid supply this device's power demand?
    pub fn can_supply(&self, device_power_w: f64) -> bool {
        device_power_w <= self.available_power_w()
    }

    /// Register device on grid
    pub fn connect_device(&mut self, device_id: u64) {
        if !self.connected_devices.contains(&device_id) {
            self.connected_devices.push(device_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_source_availability() {
        assert_eq!(EnergySource::Battery.availability_sonoran(), 1.0);
        assert!(EnergySource::Solar.availability_sonoran() > 0.8);
    }

    #[test]
    fn test_power_budget_consumption() {
        let mut budget = PowerBudget::new(1, "drone".to_string(), 500.0);
        assert_eq!(budget.state_of_charge(), 1.0);

        budget.draw_energy(50.0);
        assert!((budget.state_of_charge() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_power_budget_can_draw() {
        let budget = PowerBudget::new(1, "drone".to_string(), 500.0);
        assert!(budget.can_draw(100.0, 30.0)); // 50 Wh in 30 min
    }

    #[test]
    fn test_energy_event_calculation() {
        let event = EnergyEvent {
            timestamp_s: 0,
            device_id: 1,
            event_type: EnergyEventType::Mission,
            power_w: 100.0,
            duration_s: 3600,
        };
        let energy = event.energy_wh();
        assert_eq!(energy, 100.0); // 100W * 1 hour = 100 Wh
    }

    #[test]
    fn test_power_grid_availability() {
        let grid = PowerGrid::new(1);
        let power = grid.available_power_w();
        assert!(power > 0.0);
    }
}
