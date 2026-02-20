//! Discrete-event simulator for ecological recovery scenarios

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct SimEvent {
    pub time: f64,
    pub event_type: EventType,
}

#[derive(Debug, Clone)]
pub enum EventType {
    AgentMoves { agent_id: u64, distance_m: f64 },
    WaterApplied { zone_id: u32, liters: f64 },
    TreeSprout { zone_id: u32, count: u32 },
    Wildfire { zone_id: u32, severity: f64 },
    SensorReading { zone_id: u32, soil_health: f64 },
}

pub struct SimulationEngine {
    current_time: f64,
    event_queue: VecDeque<SimEvent>,
    zone_states: std::collections::HashMap<u32, ZoneState>,
}

#[derive(Debug, Clone)]
pub struct ZoneState {
    pub zone_id: u32,
    pub tree_density: f64,      // trees/ha
    pub soil_health: f64,       // 0-1
    pub water_content: f64,     // mm
    pub wildfire_risk: f64,     // 0-1
}

impl SimulationEngine {
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            event_queue: VecDeque::new(),
            zone_states: std::collections::HashMap::new(),
        }
    }

    pub fn enqueue_event(&mut self, event: SimEvent) {
        self.event_queue.push_back(event);
    }

    pub fn step(&mut self) -> Option<SimEvent> {
        if let Some(event) = self.event_queue.pop_front() {
            self.current_time = event.time;

            // Process event and generate consequences
            match &event.event_type {
                EventType::WaterApplied { zone_id, liters } => {
                    if let Some(zone) = self.zone_states.get_mut(zone_id) {
                        zone.water_content += liters * 0.1; // Simplified
                        zone.soil_health += 0.05;
                    }
                }
                EventType::TreeSprout { zone_id, count } => {
                    if let Some(zone) = self.zone_states.get_mut(zone_id) {
                        zone.tree_density += *count as f64 / 100.0;
                    }
                }
                EventType::Wildfire { zone_id, severity } => {
                    if let Some(zone) = self.zone_states.get_mut(zone_id) {
                        zone.tree_density *= (1.0 - severity).max(0.0);
                        zone.soil_health *= 0.6;
                    }
                }
                _ => {}
            }

            Some(event)
        } else {
            None
        }
    }

    pub fn run(&mut self, max_time: f64) -> usize {
        let mut event_count = 0;
        while self.current_time < max_time {
            if self.step().is_some() {
                event_count += 1;
            } else {
                break;
            }
        }
        event_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_step() {
        let mut sim = SimulationEngine::new();
        let event = SimEvent {
            time: 0.0,
            event_type: EventType::TreeSprout {
                zone_id: 1,
                count: 100,
            },
        };
        sim.enqueue_event(event);
        let result = sim.step();
        assert!(result.is_some());
    }
}
