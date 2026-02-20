//! Nanobot/drone agent with neuromorphic decision-making
//! Decoupled from biology; pure hardware-only cognition

use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum AgentType {
    Nanobot,      // Soil remediation
    Drone,        // Aerial survey/delivery
    Sensor,       // Environmental monitoring
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub id: u64,
    pub agent_type: AgentType,
    pub position: (f64, f64, f64),  // (lat, lon, alt)
    pub energy: f64,                 // Joules
    pub mission_queue: Vec<String>,
    pub health: f64,                 // 0.0 = failure, 1.0 = nominal
}

impl AgentState {
    pub fn new(id: u64, agent_type: AgentType) -> Self {
        Self {
            id,
            agent_type,
            position: (0.0, 0.0, 0.0),
            energy: match agent_type {
                AgentType::Nanobot => 100.0,     // Low energy
                AgentType::Drone => 50000.0,     // High energy
                AgentType::Sensor => 10000.0,    // Medium
            },
            mission_queue: Vec::new(),
            health: 1.0,
        }
    }

    pub fn energy_cost_for_distance(&self, distance_m: f64) -> f64 {
        match self.agent_type {
            AgentType::Nanobot => distance_m * 0.1,      // Low power
            AgentType::Drone => distance_m * 0.5,        // Medium power
            AgentType::Sensor => distance_m * 0.05,      // Minimal
        }
    }

    pub fn can_move(&self) -> bool {
        self.energy > 5.0 && self.health > 0.3
    }
}

/// Spiking neural network decision layer (simplified)
pub struct NeuralDecisionLayer {
    threshold: f64,
    weights: Vec<f64>,
}

impl NeuralDecisionLayer {
    pub fn new() -> Self {
        Self {
            threshold: 0.5,
            weights: vec![0.3, 0.4, 0.3],  // Input importance
        }
    }

    /// Compute decision based on sensor inputs
    /// Returns: (go_mission: bool, confidence: f64)
    pub fn decide(
        &self,
        soil_health: f64,
        water_availability: f64,
        threat_level: f64,
    ) -> (bool, f64) {
        let mut spike_count = 0.0;

        // Fire threat suppresses mission
        if threat_level > 0.7 {
            return (false, 1.0 - threat_level);
        }

        // Integrate inputs
        spike_count += self.weights[0] * soil_health;
        spike_count += self.weights[1] * water_availability;
        spike_count += (1.0 - self.weights[2]) * threat_level;

        let go = spike_count > self.threshold;
        let confidence = spike_count.abs();

        (go, confidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_energy() {
        let agent = AgentState::new(1, AgentType::Drone);
        let cost = agent.energy_cost_for_distance(1000.0);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_neural_decision() {
        let layer = NeuralDecisionLayer::new();
        let (should_go, conf) = layer.decide(0.8, 0.9, 0.2);
        assert!(should_go);
        assert!(conf > 0.0);
    }
}
