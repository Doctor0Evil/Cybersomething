//! Hardware domain models (drones, nanobots, sensors, actuators)

use serde::{Deserialize, Serialize};

/// Drone platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneType {
    Quadcopter,     // Multi-rotor, 30-60 min flight time
    FixedWing,      // Efficient surveying, 60-120 min flight time
    HybridVTOL,     // Vertical takeoff, long range
}

impl DroneType {
    /// Maximum flight time in minutes
    pub fn max_flight_time_minutes(&self) -> u32 {
        match self {
            Self::Quadcopter => 45,
            Self::FixedWing => 90,
            Self::HybridVTOL => 120,
        }
    }

    /// Cruise speed in m/s
    pub fn cruise_speed_mps(&self) -> f64 {
        match self {
            Self::Quadcopter => 12.0,
            Self::FixedWing => 15.0,
            Self::HybridVTOL => 14.0,
        }
    }

    /// Payload capacity in kg
    pub fn payload_capacity_kg(&self) -> f64 {
        match self {
            Self::Quadcopter => 2.5,
            Self::FixedWing => 5.0,
            Self::HybridVTOL => 3.5,
        }
    }
}

/// Drone platform instance with telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drone {
    pub id: u64,
    pub drone_type: DroneType,
    pub battery_percent: f64,         // 0-100%
    pub position: (f64, f64, f64),    // (lat, lon, alt_m)
    pub total_flight_time_minutes: u32,
    pub mission_cycles_completed: u32,
    pub status: DroneStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneStatus {
    Idle,
    Flying,
    OnMission,
    Charging,
    Maintenance,
    Error,
}

impl Drone {
    pub fn new(id: u64, drone_type: DroneType) -> Self {
        Self {
            id,
            drone_type,
            battery_percent: 100.0,
            position: (0.0, 0.0, 0.0),
            total_flight_time_minutes: 0,
            mission_cycles_completed: 0,
            status: DroneStatus::Idle,
        }
    }

    /// Energy cost for distance in Wh (Watt-hours)
    pub fn energy_cost_wh(&self, distance_m: f64) -> f64 {
        // Baseline: 50 Wh/km for multirotor
        let base_efficiency = match self.drone_type {
            DroneType::Quadcopter => 50.0,
            DroneType::FixedWing => 25.0,
            DroneType::HybridVTOL => 35.0,
        };

        (distance_m / 1000.0) * base_efficiency
    }

    /// Available range in meters given current battery
    pub fn available_range_m(&self) -> f64 {
        let battery_wh = self.battery_percent * 5.0; // Assume 500 Wh battery
        let efficiency = self.energy_cost_wh(1000.0);

        (battery_wh / efficiency) * 1000.0 * 0.8 // 0.8 = safety margin
    }

    pub fn can_fly(&self) -> bool {
        self.battery_percent > 20.0
            && (self.status == DroneStatus::Idle || self.status == DroneStatus::OnMission)
    }
}

/// Nanobot swarm node (cybernetic, bio-decoupled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoBot {
    pub id: u64,
    pub position: (f64, f64, f64),    // (lat, lon, depth_cm)
    pub energy_mj: f64,                // Millijoules (RF/solar harvesting)
    pub active: bool,
    pub task_queue: Vec<NanoBotTask>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NanoBotTask {
    SoilAnalysis,
    NutrientDispense,
    CompactionBreak,
    MoistureRetention,
    PhAdjustment,
}

impl NanoBotTask {
    /// Energy required in Millijoules
    pub fn energy_required_mj(&self) -> f64 {
        match self {
            Self::SoilAnalysis => 0.5,
            Self::NutrientDispense => 2.0,
            Self::CompactionBreak => 5.0,
            Self::MoistureRetention => 1.5,
            Self::PhAdjustment => 3.0,
        }
    }

    /// Time to complete in seconds
    pub fn duration_seconds(&self) -> u32 {
        match self {
            Self::SoilAnalysis => 60,
            Self::NutrientDispense => 180,
            Self::CompactionBreak => 300,
            Self::MoistureRetention => 120,
            Self::PhAdjustment => 240,
        }
    }
}

impl NanoBot {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            position: (0.0, 0.0, 0.0),
            energy_mj: 100.0,
            active: true,
            task_queue: Vec::new(),
        }
    }

    pub fn can_execute_task(&self, task: &NanoBotTask) -> bool {
        self.active && self.energy_mj >= task.energy_required_mj()
    }

    pub fn execute_task(&mut self, task: &NanoBotTask) -> bool {
        if self.can_execute_task(task) {
            self.energy_mj -= task.energy_required_mj();
            true
        } else {
            false
        }
    }

    /// Recharge from RF/solar (Millijoules added)
    pub fn recharge(&mut self, mj: f64) {
        self.energy_mj = (self.energy_mj + mj).min(100.0);
    }
}

/// Environmental sensor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sensor {
    pub id: u32,
    pub sensor_type: SensorType,
    pub location: (f64, f64, f64),
    pub last_reading: f64,
    pub accuracy_percent: f64,
    pub power_consumption_mw: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
    SoilMoisture,
    SoilPH,
    Temperature,
    Radiation,
    CO2,
    WaterFlow,
}

impl Sensor {
    pub fn new(id: u32, sensor_type: SensorType) -> Self {
        Self {
            id,
            sensor_type,
            location: (0.0, 0.0, 0.0),
            last_reading: 0.0,
            accuracy_percent: match sensor_type {
                SensorType::SoilMoisture => 2.0,
                SensorType::SoilPH => 0.1,
                SensorType::Temperature => 0.5,
                SensorType::Radiation => 3.0,
                SensorType::CO2 => 5.0,
                SensorType::WaterFlow => 2.5,
            },
            power_consumption_mw: match sensor_type {
                SensorType::SoilMoisture => 10.0,
                SensorType::SoilPH => 8.0,
                SensorType::Temperature => 5.0,
                SensorType::Radiation => 15.0,
                SensorType::CO2 => 20.0,
                SensorType::WaterFlow => 25.0,
            },
        }
    }

    /// Approximate battery life in hours (assuming 1000 mAh @ 3.7V)
    pub fn battery_life_hours(&self) -> f64 {
        let battery_energy_mwh = 1000.0 * 3.7 / 1000.0; // ~3.7 Wh = 3700 mWh
        battery_energy_mwh / self.power_consumption_mw
    }
}

/// Actuator for water or nutrient delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actuator {
    pub id: u32,
    pub actuator_type: ActuatorType,
    pub location: (f64, f64, f64),
    pub reservoir_liters: f64,
    pub delivery_rate_lpm: f64, // Liters per minute
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActuatorType {
    WaterDispenser,
    NutrientInjector,
    PestReclaimer,
}

impl Actuator {
    pub fn new(id: u32, actuator_type: ActuatorType) -> Self {
        Self {
            id,
            actuator_type,
            location: (0.0, 0.0, 0.0),
            reservoir_liters: 100.0,
            delivery_rate_lpm: match actuator_type {
                ActuatorType::WaterDispenser => 5.0,
                ActuatorType::NutrientInjector => 0.5,
                ActuatorType::PestReclaimer => 1.0,
            },
            active: false,
        }
    }

    /// Deliver substance and deplete reservoir
    pub fn deliver(&mut self, minutes: f64) -> f64 {
        let amount = (self.delivery_rate_lpm * minutes).min(self.reservoir_liters);
        self.reservoir_liters -= amount;
        amount
    }

    pub fn refill(&mut self) {
        self.reservoir_liters = 100.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drone_energy_cost() {
        let drone = Drone::new(1, DroneType::Quadcopter);
        let cost = drone.energy_cost_wh(1000.0); // 1 km
        assert!(cost > 0.0);
    }

    #[test]
    fn test_drone_available_range() {
        let mut drone = Drone::new(1, DroneType::Quadcopter);
        drone.battery_percent = 100.0;
        let range = drone.available_range_m();
        assert!(range > 10000.0); // Should be many km
    }

    #[test]
    fn test_nanobot_task_execution() {
        let mut nanobot = NanoBot::new(1);
        nanobot.energy_mj = 10.0;
        
        let can_execute = nanobot.can_execute_task(&NanoBotTask::SoilAnalysis);
        assert!(can_execute);
        
        nanobot.execute_task(&NanoBotTask::SoilAnalysis);
        assert!(nanobot.energy_mj < 10.0);
    }

    #[test]
    fn test_sensor_battery_life() {
        let sensor = Sensor::new(1, SensorType::SoilMoisture);
        let hours = sensor.battery_life_hours();
        assert!(hours > 100.0);
    }

    #[test]
    fn test_actuator_delivery() {
        let mut actuator = Actuator::new(1, ActuatorType::WaterDispenser);
        actuator.reservoir_liters = 50.0;
        
        let delivered = actuator.deliver(5.0); // 5 minutes
        assert!(delivered > 0.0);
        assert!(actuator.reservoir_liters < 50.0);
    }
}
