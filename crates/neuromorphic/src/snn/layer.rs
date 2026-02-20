//! Neural network layers (feedforward, recurrent)

use super::neuron::LIFNeuron;
use super::synapse::Synapse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single network layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralLayer {
    pub layer_id: u32,
    pub neurons: Vec<LIFNeuron>,
    pub synapses: Vec<Synapse>,
    pub input_currents: HashMap<u32, f64>, // neuron_id -> current
}

impl NeuralLayer {
    pub fn new(layer_id: u32, neuron_count: u32) -> Self {
        let neurons = (0..neuron_count)
            .map(|i| LIFNeuron::new(i))
            .collect();

        Self {
            layer_id,
            neurons,
            synapses: Vec::new(),
            input_currents: HashMap::new(),
        }
    }

    /// Add synaptic connection
    pub fn add_synapse(&mut self, synapse: Synapse) {
        self.synapses.push(synapse);
    }

    /// Set input current to a neuron
    pub fn inject_current(&mut self, neuron_id: u32, current: f64) {
        self.input_currents.insert(neuron_id, current);
    }

    /// Execute one simulation step
    pub fn step(&mut self, dt_ms: f64) -> Vec<u32> {
        // Collect spikes from previous step
        let mut spike_ids = Vec::new();

        for neuron in self.neurons.iter_mut() {
            let input = self.input_currents.get(&neuron.id).copied().unwrap_or(0.0);
            if neuron.integrate(input, dt_ms) {
                spike_ids.push(neuron.id);
            }
        }

        // Clear input currents
        self.input_currents.clear();

        spike_ids
    }

    /// Reset all neurons
    pub fn reset(&mut self) {
        for neuron in self.neurons.iter_mut() {
            neuron.reset();
        }
        self.input_currents.clear();
    }

    /// Get spike count (for monitoring activity)
    pub fn activity_level(&self) -> u32 {
        self.neurons
            .iter()
            .filter(|n| n.last_spike_time_ms > f64::NEG_INFINITY + 1.0)
            .count() as u32
    }
}

/// Multi-layer feedforward network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNNNetwork {
    pub network_id: u32,
    pub layers: Vec<NeuralLayer>,
}

impl SNNNetwork {
    pub fn new(network_id: u32) -> Self {
        Self {
            network_id,
            layers: Vec::new(),
        }
    }

    /// Add a layer to the network
    pub fn add_layer(&mut self, layer: NeuralLayer) {
        self.layers.push(layer);
    }

    /// Connect two layers
    pub fn connect_layers(
        &mut self,
        from_layer_idx: usize,
        to_layer_idx: usize,
        connection_probability: f64,
    ) {
        if from_layer_idx >= self.layers.len() || to_layer_idx >= self.layers.len() {
            return;
        }

        let from_neurons: Vec<u32> = self.layers[from_layer_idx]
            .neurons
            .iter()
            .map(|n| n.id)
            .collect();
        let to_neurons: Vec<u32> = self.layers[to_layer_idx]
            .neurons
            .iter()
            .map(|n| n.id)
            .collect();

        let mut synapse_id = 0u32;
        for &from_id in &from_neurons {
            for &to_id in &to_neurons {
                if rand::random::<f64>() < connection_probability {
                    let excitatory = rand::random::<bool>();
                    let mut syn = Synapse::new(synapse_id, from_id, to_id, excitatory);
                    syn.weight = if excitatory { 0.5 } else { -0.3 };
                    self.layers[to_layer_idx].add_synapse(syn);
                    synapse_id += 1;
                }
            }
        }
    }

    /// Run simulation for N steps
    pub fn run(&mut self, steps: u32) -> Vec<Vec<u32>> {
        let mut spike_history = Vec::new();

        for _ in 0..steps {
            let mut all_spikes = Vec::new();
            for layer in self.layers.iter_mut() {
                let layer_spikes = layer.step(1.0);
                all_spikes.extend(layer_spikes);
            }
            spike_history.push(all_spikes);
        }

        spike_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = NeuralLayer::new(0, 10);
        assert_eq!(layer.neurons.len(), 10);
    }

    #[test]
    fn test_layer_injection() {
        let mut layer = NeuralLayer::new(0, 5);
        layer.inject_current(0, 0.1);

        assert!(layer.input_currents.contains_key(&0));
    }

    #[test]
    fn test_network_creation() {
        let mut net = SNNNetwork::new(1);
        net.add_layer(NeuralLayer::new(0, 10));
        net.add_layer(NeuralLayer::new(1, 10));

        assert_eq!(net.layers.len(), 2);
    }

    #[test]
    fn test_network_run() {
        let mut net = SNNNetwork::new(1);
        net.add_layer(NeuralLayer::new(0, 10));
        
        let history = net.run(5);
        assert_eq!(history.len(), 5);
    }
}
