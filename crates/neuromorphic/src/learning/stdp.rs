//! Spike-timing-dependent plasticity (STDP)
//! Unsupervised learning rule for neuromorphic systems

use serde::{Deserialize, Serialize};

/// STDP learning parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STDPParameters {
    pub learning_rate: f64,        // 0-1
    pub positive_window_ms: f64,   // Pre before post
    pub negative_window_ms: f64,   // Post before pre
    pub time_constant_ms: f64,     // Decay
}

impl Default for STDPParameters {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            positive_window_ms: 20.0,
            negative_window_ms: 20.0,
            time_constant_ms: 20.0,
        }
    }
}

/// STDP learning rule
pub struct STDPLearner {
    pub params: STDPParameters,
}

impl STDPLearner {
    pub fn new(params: STDPParameters) -> Self {
        Self { params }
    }

    /// Compute weight change based on spike timing
    /// dt = t_post - t_pre (positive if post fires after pre)
    pub fn compute_weight_change(&self, dt_ms: f64) -> f64 {
        if dt_ms > 0.0 {
            // Post-synaptic spike after pre: LTP (potentiation)
            if dt_ms < self.params.positive_window_ms {
                let normalized_dt = dt_ms / self.params.time_constant_ms;
                self.params.learning_rate * (-normalized_dt).exp()
            } else {
                0.0
            }
        } else {
            // Pre-synaptic spike after post: LTD (depression)
            if -dt_ms < self.params.negative_window_ms {
                let normalized_dt = -dt_ms / self.params.time_constant_ms;
                -self.params.learning_rate * (-normalized_dt).exp()
            } else {
                0.0
            }
        }
    }

    /// Update synaptic weight given spike timings
    pub fn update_weight(
        &self,
        current_weight: f64,
        t_pre_ms: f64,
        t_post_ms: f64,
    ) -> f64 {
        let dt = t_post_ms - t_pre_ms;
        let dw = self.compute_weight_change(dt);
        let new_weight = current_weight + dw;
        new_weight.clamp(-1.0, 1.0)
    }
}

/// STDP learning window visualization (for debugging)
pub fn stdp_window(dt_ms: f64, params: &STDPParameters) -> f64 {
    if dt_ms > 0.0 && dt_ms < params.positive_window_ms {
        (-dt_ms / params.time_constant_ms).exp()
    } else if dt_ms < 0.0 && -dt_ms < params.negative_window_ms {
        -(-dt_ms / params.time_constant_ms).exp()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdp_ltp() {
        let params = STDPParameters::default();
        let learner = STDPLearner::new(params);

        // Post fires 10ms after pre: LTP
        let dw = learner.compute_weight_change(10.0);
        assert!(dw > 0.0);
    }

    #[test]
    fn test_stdp_ltd() {
        let params = STDPParameters::default();
        let learner = STDPLearner::new(params);

        // Pre fires 10ms after post: LTD
        let dw = learner.compute_weight_change(-10.0);
        assert!(dw < 0.0);
    }

    #[test]
    fn test_stdp_outside_window() {
        let params = STDPParameters::default();
        let learner = STDPLearner::new(params);

        let dw = learner.compute_weight_change(100.0);
        assert_eq!(dw, 0.0);
    }

    #[test]
    fn test_weight_update() {
        let params = STDPParameters::default();
        let learner = STDPLearner::new(params);

        let old_weight = 0.5;
        let new_weight = learner.update_weight(old_weight, 0.0, 10.0);

        assert!(new_weight > old_weight);
    }

    #[test]
    fn test_weight_clipping() {
        let mut params = STDPParameters::default();
        params.learning_rate = 1.0;

        let learner = STDPLearner::new(params);
        let new_weight = learner.update_weight(1.0, 0.0, 5.0);

        assert!(new_weight <= 1.0);
    }
}
