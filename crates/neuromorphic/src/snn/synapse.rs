//! Synaptic connections and transmission

use serde::{Deserialize, Serialize};

/// Synaptic connection between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub id: u32,
    pub pre_neuron_id: u32,  // Source neuron
    pub post_neuron_id: u32, // Target neuron
    pub weight: f64,         // Synaptic strength (-1.0 to +1.0)
    pub delay_ms: f64,       // Axonal delay
    pub is_excitatory: bool,
    pub trace_pre: f64,      // Pre-synaptic trace (for STDP)
    pub trace_post: f64,     // Post-synaptic trace (for STDP)
}

impl Synapse {
    pub fn new(id: u32, pre: u32, post: u32, excitatory: bool) -> Self {
        Self {
            id,
            pre_neuron_id: pre,
            post_neuron_id: post,
            weight: 0.5,
            delay_ms: 1.0,
            is_excitatory: excitatory,
            trace_pre: 0.0,
            trace_post: 0.0,
        }
    }

    /// Transmit spike across synapse to post-synaptic neuron
    pub fn transmit(&self) -> f64 {
        let sign = if self.is_excitatory { 1.0 } else { -1.0 };
        sign * self.weight
    }

    /// Update synaptic traces (exponential decay for STDP)
    pub fn decay_traces(&mut self, dt_ms: f64, tau_ms: f64) {
        let decay = (-dt_ms / tau_ms).exp();
        self.trace_pre *= decay;
        self.trace_post *= decay;
    }

    /// Mark pre-synaptic spike
    pub fn mark_pre_spike(&mut self) {
        self.trace_pre = 1.0;
    }

    /// Mark post-synaptic spike
    pub fn mark_post_spike(&mut self) {
        self.trace_post = 1.0;
    }

    /// Clip weight to valid range
    pub fn clip_weight(&mut self) {
        self.weight = self.weight.clamp(-1.0, 1.0);
    }
}

/// Synaptic delay line (event-based spike queue)
pub struct DelayLine {
    pub delay_ms: f64,
    pub current_time_ms: f64,
    pub events: std::collections::VecDeque<(f64, f64)>, // (time, current)
}

impl DelayLine {
    pub fn new(delay_ms: f64) -> Self {
        Self {
            delay_ms,
            current_time_ms: 0.0,
            events: std::collections::VecDeque::new(),
        }
    }

    /// Enqueue a spike event
    pub fn enqueue(&mut self, current: f64) {
        self.events.push_back((self.current_time_ms + self.delay_ms, current));
    }

    /// Dequeue and deliver spikes at current time
    pub fn deliver(&mut self, current_time_ms: f64) -> f64 {
        self.current_time_ms = current_time_ms;
        let mut total_current = 0.0;

        while let Some(&(spike_time, current)) = self.events.front() {
            if spike_time <= current_time_ms {
                self.events.pop_front();
                total_current += current;
            } else {
                break;
            }
        }

        total_current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synapse_creation() {
        let syn = Synapse::new(1, 1, 2, true);
        assert_eq!(syn.pre_neuron_id, 1);
        assert_eq!(syn.post_neuron_id, 2);
        assert!(syn.is_excitatory);
    }

    #[test]
    fn test_synapse_transmit() {
        let syn = Synapse::new(1, 1, 2, true);
        let current = syn.transmit();
        assert!(current > 0.0);
    }

    #[test]
    fn test_synapse_inhibitory() {
        let mut syn = Synapse::new(1, 1, 2, false);
        syn.weight = 0.8;
        let current = syn.transmit();
        assert!(current < 0.0);
    }

    #[test]
    fn test_delay_line() {
        let mut line = DelayLine::new(5.0);
        line.enqueue(0.1);

        let delivered_at_0 = line.deliver(0.0);
        assert_eq!(delivered_at_0, 0.0);

        let delivered_at_5 = line.deliver(5.0);
        assert!(delivered_at_5 > 0.0);
    }

    #[test]
    fn test_synapse_trace_decay() {
        let mut syn = Synapse::new(1, 1, 2, true);
        syn.trace_pre = 1.0;
        syn.trace_post = 1.0;

        syn.decay_traces(1.0, 10.0);
        assert!(syn.trace_pre < 1.0);
        assert!(syn.trace_post < 1.0);
    }
}
