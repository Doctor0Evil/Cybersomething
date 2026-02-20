//! Reward-based learning for ecological goals

use serde::{Deserialize, Serialize};

/// Reward signal (ecological outcome)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RewardSignal {
    TreeGrowth(f64),              // Positive: trees established
    SoilHealthImprovement(f64),   // Positive: soil metrics improve
    WildlifeReturn(f64),          // Positive: species detected
    WaterConservation(f64),       // Positive: efficient use
    FireRiskReduction(f64),       // Positive: defensible space
    Penalty(f64),                 // Negative: failed task
}

impl RewardSignal {
    /// Extract numeric reward value
    pub fn value(&self) -> f64 {
        match self {
            Self::TreeGrowth(v) => *v,
            Self::SoilHealthImprovement(v) => *v,
            Self::WildlifeReturn(v) => *v,
            Self::WaterConservation(v) => *v,
            Self::FireRiskReduction(v) => *v,
            Self::Penalty(v) => -*v,
        }
    }
}

/// Reward-driven agent learning state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardLearner {
    pub agent_id: u64,
    pub cumulative_reward: f64,
    pub episode_rewards: Vec<f64>,
    pub value_estimate: f64,       // V(s) estimate
    pub learning_rate: f64,
    pub discount_factor: f64,      // γ (gamma)
}

impl RewardLearner {
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            cumulative_reward: 0.0,
            episode_rewards: Vec::new(),
            value_estimate: 0.5,
            learning_rate: 0.1,
            discount_factor: 0.99,
        }
    }

    /// Receive reward and update value estimate
    pub fn receive_reward(&mut self, reward: RewardSignal) {
        let r = reward.value();
        self.cumulative_reward += r;
        self.episode_rewards.push(r);

        // Temporal-difference learning: V(s) ← V(s) + α[R + γV(s') - V(s)]
        let td_error = r + self.discount_factor * self.value_estimate - self.value_estimate;
        self.value_estimate += self.learning_rate * td_error;

        tracing::debug!(
            agent_id = self.agent_id,
            reward = r,
            value_estimate = self.value_estimate,
            "Reward received"
        );
    }

    /// Get average reward per episode
    pub fn average_episode_reward(&self) -> f64 {
        if self.episode_rewards.is_empty() {
            0.0
        } else {
            self.episode_rewards.iter().sum::<f64>() / self.episode_rewards.len() as f64
        }
    }

    /// Compute advantage (A(s,a) = Q(s,a) - V(s))
    pub fn advantage(&self, action_value: f64) -> f64 {
        action_value - self.value_estimate
    }

    /// Reset episode rewards
    pub fn start_new_episode(&mut self) {
        self.episode_rewards.clear();
    }
}

/// Swarm reward aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRewardPool {
    pub pool_id: u32,
    pub individual_learners: std::collections::HashMap<u64, RewardLearner>,
    pub collective_reward: f64,
}

impl SwarmRewardPool {
    pub fn new(pool_id: u32) -> Self {
        Self {
            pool_id,
            individual_learners: std::collections::HashMap::new(),
            collective_reward: 0.0,
        }
    }

    /// Register agent learner
    pub fn register_learner(&mut self, learner: RewardLearner) {
        self.individual_learners.insert(learner.agent_id, learner);
    }

    /// Distribute shared reward to all learners (cooperative)
    pub fn distribute_shared_reward(&mut self, reward: RewardSignal) {
        let r = reward.value();
        self.collective_reward += r;

        // Each agent gets a share
        for learner in self.individual_learners.values_mut() {
            let shared_reward = RewardSignal::TreeGrowth(r / self.individual_learners.len() as f64);
            learner.receive_reward(shared_reward);
        }
    }

    /// Individual reward allocation
    pub fn reward_agent(&mut self, agent_id: u64, reward: RewardSignal) {
        if let Some(learner) = self.individual_learners.get_mut(&agent_id) {
            learner.receive_reward(reward);
        }
    }

    /// Average performance across swarm
    pub fn average_value_estimate(&self) -> f64 {
        if self.individual_learners.is_empty() {
            0.0
        } else {
            let sum: f64 = self
                .individual_learners
                .values()
                .map(|l| l.value_estimate)
                .sum();
            sum / self.individual_learners.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_signal() {
        let reward = RewardSignal::TreeGrowth(5.0);
        assert_eq!(reward.value(), 5.0);

        let penalty = RewardSignal::Penalty(2.0);
        assert_eq!(penalty.value(), -2.0);
    }

    #[test]
    fn test_reward_learner_creation() {
        let learner = RewardLearner::new(1);
        assert_eq!(learner.agent_id, 1);
        assert_eq!(learner.cumulative_reward, 0.0);
    }

    #[test]
    fn test_reward_learner_update() {
        let mut learner = RewardLearner::new(1);
        let old_value = learner.value_estimate;

        learner.receive_reward(RewardSignal::TreeGrowth(1.0));

        assert_ne!(learner.value_estimate, old_value);
    }

    #[test]
    fn test_advantage_calculation() {
        let learner = RewardLearner::new(1);
        let adv = learner.advantage(0.8);
        assert!(adv < 0.3);
    }

    #[test]
    fn test_swarm_reward_pool() {
        let mut pool = SwarmRewardPool::new(1);

        let learner1 = RewardLearner::new(1);
        let learner2 = RewardLearner::new(2);

        pool.register_learner(learner1);
        pool.register_learner(learner2);

        pool.distribute_shared_reward(RewardSignal::TreeGrowth(10.0));

        assert!(pool.collective_reward > 0.0);
    }
}
