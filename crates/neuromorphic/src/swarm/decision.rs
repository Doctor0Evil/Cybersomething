//! Integrated decision-making for swarms

use super::agent::SwarmAgent;
use super::collective::SwarmCollective;
use serde::{Deserialize, Serialize};

/// Mission objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionObjective {
    pub objective_id: u32,
    pub objective_type: ObjectiveType,
    pub target_zone_id: u32,
    pub urgency: f64,              // 0-1
    pub resources_required: u32,
    pub deadline_seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    Survey,
    WaterDelivery,
    SoilAmendment,
    WildlifeMonitoring,
    FireSuppressionPrep,
}

/// Decision-making system for swarm missions
pub struct SwarmDecisionSystem {
    pub objectives: Vec<MissionObjective>,
    pub active_collective: Option<SwarmCollective>,
}

impl SwarmDecisionSystem {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            active_collective: None,
        }
    }

    /// Add mission objective
    pub fn add_objective(&mut self, objective: MissionObjective) {
        self.objectives.push(objective);
    }

    /// Rank objectives by urgency and agent capacity
    pub fn prioritize_objectives(&self) -> Vec<&MissionObjective> {
        let mut ranked = self.objectives.iter().collect::<Vec<_>>();
        ranked.sort_by(|a, b| {
            b.urgency
                .partial_cmp(&a.urgency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }

    /// Allocate agents to objectives
    pub fn allocate_agents(&self, collective: &SwarmCollective) -> std::collections::HashMap<u64, MissionObjective> {
        let mut allocation = std::collections::HashMap::new();
        let priorities = self.prioritize_objectives();

        let agent_ids: Vec<u64> = collective.agents.keys().copied().collect();
        let mut agent_idx = 0;

        for objective in priorities {
            let agents_needed = objective.resources_required as usize;
            for _ in 0..agents_needed {
                if agent_idx < agent_ids.len() {
                    allocation.insert(agent_ids[agent_idx], objective.clone());
                    agent_idx += 1;
                } else {
                    break;
                }
            }
        }

        allocation
    }

    /// Evaluate mission success probability
    pub fn mission_feasibility(&self, collective: &SwarmCollective, objective: &MissionObjective) -> f64 {
        if collective.agents.is_empty() {
            return 0.0;
        }

        let agent_count = collective.agents.len() as f64;
        let resources_factor = (agent_count / objective.resources_required as f64).min(1.0);
        let cohesion_factor = collective.coordination_state.group_cohesion;
        let arousal_factor = collective.coordination_state.average_arousal;

        (resources_factor * 0.4 + cohesion_factor * 0.3 + arousal_factor * 0.3).min(1.0)
    }

    /// Adaptive mission re-planning based on swarm state
    pub fn replan_if_needed(&mut self, collective: &SwarmCollective) {
        let low_cohesion = collective.coordination_state.group_cohesion < 0.3;
        let low_arousal = collective.coordination_state.average_arousal < 0.2;
        let threats = collective
            .agents
            .values()
            .any(|a| a.local_sensor_data.threats_detected > 0);

        if low_cohesion || low_arousal || threats {
            // Pause non-urgent objectives
            self.objectives.iter_mut().for_each(|obj| {
                if obj.urgency < 0.5 {
                    obj.urgency *= 0.5;
                }
            });
        }
    }

    /// Get next action for an agent
    pub fn next_action_for_agent(
        &self,
        agent: &SwarmAgent,
        mission_allocation: &std::collections::HashMap<u64, MissionObjective>,
    ) -> SwarmAction {
        if let Some(objective) = mission_allocation.get(&agent.id) {
            match objective.objective_type {
                ObjectiveType::Survey => SwarmAction::Survey,
                ObjectiveType::WaterDelivery => SwarmAction::Deliver,
                ObjectiveType::SoilAmendment => SwarmAction::Amend,
                ObjectiveType::WildlifeMonitoring => SwarmAction::Monitor,
                ObjectiveType::FireSuppressionPrep => SwarmAction::Retreat,
            }
        } else {
            SwarmAction::Explore
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmAction {
    Survey,
    Deliver,
    Amend,
    Monitor,
    Retreat,
    Explore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_creation() {
        let mission = MissionObjective {
            objective_id: 1,
            objective_type: ObjectiveType::Survey,
            target_zone_id: 100,
            urgency: 0.8,
            resources_required: 5,
            deadline_seconds: 3600,
        };
        assert_eq!(mission.urgency, 0.8);
    }

    #[test]
    fn test_decision_system_creation() {
        let system = SwarmDecisionSystem::new();
        assert!(system.objectives.is_empty());
    }

    #[test]
    fn test_objective_prioritization() {
        let mut system = SwarmDecisionSystem::new();

        system.add_objective(MissionObjective {
            objective_id: 1,
            objective_type: ObjectiveType::Survey,
            target_zone_id: 100,
            urgency: 0.3,
            resources_required: 2,
            deadline_seconds: 3600,
        });

        system.add_objective(MissionObjective {
            objective_id: 2,
            objective_type: ObjectiveType::FireSuppressionPrep,
            target_zone_id: 101,
            urgency: 0.9,
            resources_required: 10,
            deadline_seconds: 600,
        });

        let ranked = system.prioritize_objectives();
        assert_eq!(ranked[0].objective_id, 2); // Fire suppression first
    }
}
