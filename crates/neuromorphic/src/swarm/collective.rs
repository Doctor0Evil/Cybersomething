//! Collective swarm behaviors and consensus algorithms

use super::agent::SwarmAgent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Swarm collective consensus state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmCollective {
    pub swarm_id: u32,
    pub agents: HashMap<u64, SwarmAgent>,
    pub consensus_decision: ConsensusDecision,
    pub coordination_state: CoordinationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusDecision {
    Explore,
    Concentrate,
    Retreat,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationState {
    pub centroid_lat: f64,
    pub centroid_lon: f64,
    pub average_arousal: f64,
    pub group_cohesion: f64,      // 0-1
    pub time_since_decision_s: u32,
}

impl SwarmCollective {
    pub fn new(swarm_id: u32) -> Self {
        Self {
            swarm_id,
            agents: HashMap::new(),
            consensus_decision: ConsensusDecision::Explore,
            coordination_state: CoordinationState {
                centroid_lat: 0.0,
                centroid_lon: 0.0,
                average_arousal: 0.5,
                group_cohesion: 0.5,
                time_since_decision_s: 0,
            },
        }
    }

    /// Add agent to swarm
    pub fn add_agent(&mut self, agent: SwarmAgent) {
        self.agents.insert(agent.id, agent);
    }

    /// Calculate swarm centroid
    pub fn update_centroid(&mut self) {
        if self.agents.is_empty() {
            return;
        }

        let mut sum_lat = 0.0;
        let mut sum_lon = 0.0;
        let mut sum_arousal = 0.0;

        for agent in self.agents.values() {
            sum_lat += agent.position.0;
            sum_lon += agent.position.1;
            sum_arousal += agent.snn_state.arousal_level;
        }

        let n = self.agents.len() as f64;
        self.coordination_state.centroid_lat = sum_lat / n;
        self.coordination_state.centroid_lon = sum_lon / n;
        self.coordination_state.average_arousal = sum_arousal / n;
    }

    /// Quorum-based consensus (Majority Voting)
    pub fn consensus_majority(&mut self) {
        let mut explore_count = 0;
        let mut concentrate_count = 0;
        let mut retreat_count = 0;

        for agent in self.agents.values() {
            match agent.snn_state.task_priority {
                p if p > 0.7 => concentrate_count += 1,
                p if p < 0.3 => retreat_count += 1,
                _ => explore_count += 1,
            }
        }

        self.consensus_decision = match std::cmp::max(
            std::cmp::max(explore_count, concentrate_count),
            retreat_count,
        ) {
            c if c == explore_count => ConsensusDecision::Explore,
            c if c == concentrate_count => ConsensusDecision::Concentrate,
            c if c == retreat_count => ConsensusDecision::Retreat,
            _ => ConsensusDecision::Wait,
        };
    }

    /// Cohesion metric (distance to centroid variance)
    pub fn calculate_cohesion(&mut self) {
        if self.agents.is_empty() {
            self.coordination_state.group_cohesion = 0.0;
            return;
        }

        let mut distance_sum = 0.0;
        for agent in self.agents.values() {
            let dlat = agent.position.0 - self.coordination_state.centroid_lat;
            let dlon = agent.position.1 - self.coordination_state.centroid_lon;
            let dist = (dlat * dlat + dlon * dlon).sqrt();
            distance_sum += dist;
        }

        let avg_distance = distance_sum / self.agents.len() as f64;
        // Closer = more cohesion
        self.coordination_state.group_cohesion = (-avg_distance * 10.0).exp().clamp(0.0, 1.0);
    }

    /// Flocking rule: local alignment with neighbors
    pub fn local_alignment(&mut self, neighbor_radius_km: f64) {
        let agent_ids: Vec<u64> = self.agents.keys().copied().collect();

        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                let mut avg_heading = agent.heading;
                let mut neighbor_count = 0;

                for other_agent in self.agents.values() {
                    if other_agent.id != agent_id {
                        let dlat = other_agent.position.0 - agent.position.0;
                        let dlon = other_agent.position.1 - agent.position.1;
                        let dist_km = (dlat * dlat + dlon * dlon).sqrt() * 111.0;

                        if dist_km < neighbor_radius_km {
                            avg_heading += other_agent.heading;
                            neighbor_count += 1;
                        }
                    }
                }

                if neighbor_count > 0 {
                    avg_heading /= (neighbor_count + 1) as f64;
                    agent.heading = 0.8 * agent.heading + 0.2 * avg_heading;
                }
            }
        }
    }

    /// Separation rule: maintain minimum distance
    pub fn local_separation(&mut self, min_distance_km: f64) {
        let agent_ids: Vec<u64> = self.agents.keys().copied().collect();

        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                let mut repulsion_lat = 0.0;
                let mut repulsion_lon = 0.0;

                for other_agent in self.agents.values() {
                    if other_agent.id != agent_id {
                        let dlat = agent.position.0 - other_agent.position.0;
                        let dlon = agent.position.1 - other_agent.position.1;
                        let dist_km = (dlat * dlat + dlon * dlon).sqrt() * 111.0;

                        if dist_km < min_distance_km && dist_km > 0.001 {
                            repulsion_lat += dlat / (dist_km * dist_km);
                            repulsion_lon += dlon / (dist_km * dist_km);
                        }
                    }
                }

                agent.velocity.0 += repulsion_lat * 0.1;
                agent.velocity.1 += repulsion_lon * 0.1;
            }
        }
    }

    /// Simulate one timestep for all agents
    pub fn step(&mut self, dt_seconds: f64) {
        for agent in self.agents.values_mut() {
            agent.move_agent(dt_seconds);
        }

        self.update_centroid();
        self.calculate_cohesion();
        self.consensus_majority();

        self.coordination_state.time_since_decision_s += dt_seconds as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::agent::SwarmAgentType;

    #[test]
    fn test_collective_creation() {
        let collective = SwarmCollective::new(1);
        assert_eq!(collective.swarm_id, 1);
        assert!(collective.agents.is_empty());
    }

    #[test]
    fn test_add_agent() {
        let mut collective = SwarmCollective::new(1);
        let agent = SwarmAgent::new(1, SwarmAgentType::Drone);
        collective.add_agent(agent);

        assert_eq!(collective.agents.len(), 1);
    }

    #[test]
    fn test_centroid_calculation() {
        let mut collective = SwarmCollective::new(1);
        
        let mut agent1 = SwarmAgent::new(1, SwarmAgentType::Drone);
        agent1.position = (33.0, -112.0, 100.0);
        
        let mut agent2 = SwarmAgent::new(2, SwarmAgentType::Drone);
        agent2.position = (33.1, -112.1, 100.0);

        collective.add_agent(agent1);
        collective.add_agent(agent2);
        collective.update_centroid();

        assert!((collective.coordination_state.centroid_lat - 33.05).abs() < 0.01);
    }

    #[test]
    fn test_consensus() {
        let mut collective = SwarmCollective::new(1);
        
        let agent1 = SwarmAgent::new(1, SwarmAgentType::Drone);
        let agent2 = SwarmAgent::new(2, SwarmAgentType::Drone);
        
        collective.add_agent(agent1);
        collective.add_agent(agent2);
        collective.consensus_majority();

        assert_ne!(collective.consensus_decision, ConsensusDecision::Wait);
    }
}
