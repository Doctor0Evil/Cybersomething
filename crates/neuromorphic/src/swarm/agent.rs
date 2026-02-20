//! Individual swarm agent with neuromorphic cognition

use cybersomething_core::models::*;
use serde::{Deserialize, Serialize};

/// Swarm agent (drone or nanobot) with integrated SNN decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgent {
    pub id: u64,
    pub agent_type: SwarmAgentType,
    pub position: (f64, f64, f64),    // (lat, lon, depth_or_alt)
    pub velocity: (f64, f64, f64),    // m/s
    pub heading: f64,                  // degrees
    pub state: AgentState,
    pub local_sensor_data: SensorReadings,
    pub snn_state: SNNAgentState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmAgentType {
    Drone,
    Nanobot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Exploring,
    ExecutingTask,
    Returning,
    Communicating,
    Error,
}

/// Local sensor readings (environmental context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReadings {
    pub soil_moisture_percent: f64,
    pub soil_ph: f64,
    pub temperature_c: f64,
    pub light_level: f64,           // 0-1
    pub co2_ppm: f64,
    pub threats_detected: u32,      // Wildfire, etc.
}

impl Default for SensorReadings {
    fn default() -> Self {
        Self {
            soil_moisture_percent: 20.0,
            soil_ph: 7.2,
            temperature_c: 25.0,
            light_level: 0.5,
            co2_ppm: 400.0,
            threats_detected: 0,
        }
    }
}

/// SNN internal state for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNNAgentState {
    pub decision_confidence: f64,       // 0-1
    pub task_priority: f64,             // 0-1 (0=idle, 1=urgent)
    pub arousal_level: f64,             // 0-1 (energy, alertness)
    pub social_influence: f64,          // 0-1 (swarming cohesion)
    pub recent_spikes: u32,             // Activity level
}

impl Default for SNNAgentState {
    fn default() -> Self {
        Self {
            decision_confidence: 0.5,
            task_priority: 0.3,
            arousal_level: 0.7,
            social_influence: 0.5,
            recent_spikes: 0,
        }
    }
}

impl SwarmAgent {
    pub fn new(id: u64, agent_type: SwarmAgentType) -> Self {
        Self {
            id,
            agent_type,
            position: (0.0, 0.0, 0.0),
            velocity: (0.0, 0.0, 0.0),
            heading: 0.0,
            state: AgentState::Idle,
            local_sensor_data: SensorReadings::default(),
            snn_state: SNNAgentState::default(),
        }
    }

    /// Update agent position based on velocity
    pub fn move_agent(&mut self, dt_seconds: f64) {
        self.position.0 += self.velocity.0 * dt_seconds / 111000.0; // deg/meter conversion
        self.position.1 += self.velocity.1 * dt_seconds / 111000.0;
        self.position.2 += self.velocity.2 * dt_seconds;
    }

    /// Make neuromorphic decision based on sensor data
    pub fn snn_decision(&mut self) -> AgentAction {
        // Simplified SNN decision: weighted sum of sensor inputs
        let mut decision_potential = 0.0;

        // High soil moisture → reduce irrigation task
        decision_potential -= (self.local_sensor_data.soil_moisture_percent / 100.0) * 0.3;

        // Low soil pH → prioritize amendment
        decision_potential += ((7.5 - self.local_sensor_data.soil_ph) / 7.5) * 0.2;

        // Threats detected → retreat
        if self.local_sensor_data.threats_detected > 0 {
            decision_potential -= 0.8;
            self.snn_state.arousal_level = 0.95;
        }

        // Low temperature → reduce activity
        if self.local_sensor_data.temperature_c < 10.0 {
            decision_potential -= 0.2;
            self.snn_state.arousal_level *= 0.7;
        }

        // Update SNN state
        self.snn_state.task_priority = decision_potential.clamp(0.0, 1.0);
        self.snn_state.recent_spikes = (decision_potential.abs() * 100.0) as u32;

        // Convert to action
        self.decision_to_action()
    }

    /// Convert decision potential to concrete action
    fn decision_to_action(&self) -> AgentAction {
        match self.snn_state.task_priority {
            p if p > 0.8 => AgentAction::ExecuteTask,
            p if p > 0.5 => AgentAction::Explore,
            p if p > 0.2 => AgentAction::Monitor,
            _ => AgentAction::ReturnHome,
        }
    }

    /// Update heading toward target
    pub fn move_toward(&mut self, target_lat: f64, target_lon: f64) {
        let dlat = target_lat - self.position.0;
        let dlon = target_lon - self.position.1;
        self.heading = dlon.atan2(dlat).to_degrees();

        let distance = ((dlat * dlat + dlon * dlon).sqrt()) * 111000.0; // meters
        let speed = match self.agent_type {
            SwarmAgentType::Drone => 12.0,
            SwarmAgentType::Nanobot => 0.1,
        };

        if distance > 1.0 {
            self.velocity.0 = (self.heading.to_radians().sin()) * speed;
            self.velocity.1 = (self.heading.to_radians().cos()) * speed;
            self.state = AgentState::Exploring;
        } else {
            self.velocity = (0.0, 0.0, 0.0);
            self.state = AgentState::ExecutingTask;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAction {
    ExecuteTask,
    Explore,
    Monitor,
    ReturnHome,
    Idle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = SwarmAgent::new(1, SwarmAgentType::Drone);
        assert_eq!(agent.id, 1);
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[test]
    fn test_agent_movement() {
        let mut agent = SwarmAgent::new(1, SwarmAgentType::Drone);
        agent.velocity = (1.0, 0.0, 0.0);
        
        agent.move_agent(10.0);
        assert!(agent.position.0 > 0.0);
    }

    #[test]
    fn test_snn_decision() {
        let mut agent = SwarmAgent::new(1, SwarmAgentType::Drone);
        agent.local_sensor_data.soil_moisture_percent = 80.0;
        
        let action = agent.snn_decision();
        assert_eq!(action, AgentAction::ReturnHome);
    }

    #[test]
    fn test_move_toward() {
        let mut agent = SwarmAgent::new(1, SwarmAgentType::Drone);
        agent.move_toward(33.5, -112.0);
        
        assert_ne!(agent.velocity.0, 0.0);
    }
}
