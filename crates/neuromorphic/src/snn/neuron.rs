//! Spiking neuron models (Leaky Integrate-and-Fire)
//! Hardware-only computational units

use serde::{Deserialize, Serialize};

/// Leaky Integrate-and-Fire (LIF) neuron
/// Simplified hardware-implementable model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIFNeuron {
    pub id: u32,
    pub membrane_potential: f64,    // Volts (normalized -70 to +30 mV)
    pub threshold: f64,              // Spike threshold (20 mV)
    pub rest_potential: f64,         // Resting potential (-70 mV)
    pub leak_conductance: f64,       // Conductance (siemens)
    pub time_constant_ms: f64,       // Membrane time constant
    pub refractory_period_ms: f64,   // AHP refractory period
    pub last_spike_time_ms: f64,     // For refractory tracking
}

impl LIFNeuron {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            membrane_potential: -0.7,  // Normalized: -70mV
            threshold: 0.2,            // 20mV
            rest_potential: -0.7,      // -70mV
            leak_conductance: 0.1,     // 100 pS
            time_constant_ms: 20.0,
            refractory_period_ms: 2.0,
            last_spike_time_ms: f64::NEG_INFINITY,
        }
    }

    /// Update membrane potential given input current (Amperes)
    /// Uses Euler integration step
    pub fn integrate(&mut self, input_current_a: f64, dt_ms: f64) -> bool {
        let elapsed = self.last_spike_time_ms;
        let now_ms = elapsed + dt_ms;

        // Check refractory period
        if (now_ms - self.last_spike_time_ms) < self.refractory_period_ms {
            // Still refractory: hyperpolarize slightly
            self.membrane_potential = self.rest_potential - 0.05;
            return false;
        }

        // Leaky integration: dV/dt = -g_leak * (V - E_leak) + I
        let driving_force = self.membrane_potential - self.rest_potential;
        let leak_current = self.leak_conductance * driving_force;
        let dv_dt = (-leak_current + input_current_a) / 20.0; // Simplified capacitance

        // Euler step
        let dt_s = dt_ms / 1000.0;
        self.membrane_potential += dv_dt * dt_s;

        // Check spike condition
        if self.membrane_potential > self.threshold {
            self.membrane_potential = self.rest_potential;
            self.last_spike_time_ms = now_ms;
            return true;
        }

        false
    }

    /// Reset to resting state
    pub fn reset(&mut self) {
        self.membrane_potential = self.rest_potential;
        self.last_spike_time_ms = f64::NEG_INFINITY;
    }

    /// Is neuron currently in refractory period?
    pub fn in_refractory(&self, current_time_ms: f64) -> bool {
        (current_time_ms - self.last_spike_time_ms) < self.refractory_period_ms
    }
}

/// Poisson spike generator (for testing/stimulus)
pub struct PoissonGenerator {
    pub rate_hz: f64,
    pub last_spike_ms: f64,
}

impl PoissonGenerator {
    pub fn new(rate_hz: f64) -> Self {
        Self {
            rate_hz,
            last_spike_ms: 0.0,
        }
    }

    /// Generate spike with probability based on rate
    pub fn spike(&mut self, dt_ms: f64) -> bool {
        let prob = self.rate_hz / 1000.0 * dt_ms; // Probability in this dt
        rand::random::<f64>() < prob
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lif_neuron_creation() {
        let neuron = LIFNeuron::new(1);
        assert_eq!(neuron.id, 1);
        assert!(neuron.membrane_potential < neuron.threshold);
    }

    #[test]
    fn test_lif_neuron_spike() {
        let mut neuron = LIFNeuron::new(1);
        
        // Strong input current
        let mut spiked = false;
        for _ in 0..100 {
            spiked = neuron.integrate(0.1, 1.0); // 100 nA current
            if spiked {
                break;
            }
        }

        assert!(spiked, "Neuron should spike with strong input");
        assert!(neuron.membrane_potential < neuron.threshold);
    }

    #[test]
    fn test_lif_neuron_refractory() {
        let mut neuron = LIFNeuron::new(1);
        neuron.last_spike_time_ms = 0.0;

        assert!(neuron.in_refractory(1.0)); // 1ms after spike
        assert!(!neuron.in_refractory(5.0)); // 5ms after spike
    }

    #[test]
    fn test_poisson_generator() {
        let mut gen = PoissonGenerator::new(100.0); // 100 Hz
        let mut spike_count = 0;

        for _ in 0..1000 {
            if gen.spike(1.0) {
                spike_count += 1;
            }
        }

        // Should be roughly 100 spikes (with variance)
        assert!(spike_count > 50 && spike_count < 150);
    }
}
