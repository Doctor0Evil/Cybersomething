//! Cybersomething Neuromorphic Intelligence
//!
//! Spiking neural networks (SNN) for distributed decision-making in swarms.
//! Completely hardware-based cognition, decoupled from biology.
//!
//! # Modules
//!
//! - `snn` — Spiking neuron models and layers
//! - `swarm` — Collective agent behaviors and consensus
//! - `learning` — Spike-timing-dependent plasticity (STDP) and reward-based adaptation

pub mod snn;
pub mod swarm;
pub mod learning;

pub use snn::*;
pub use swarm::*;
pub use learning::*;
