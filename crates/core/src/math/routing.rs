//! Routing algorithms for swarm agents and drones

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// Waypoint for a mission route
#[derive(Debug, Clone)]
pub struct Waypoint {
    pub zone_id: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f64,
    pub action: Option<String>, // e.g., "water_500L", "survey"
}

/// Route segment with cost
#[derive(Clone, Eq, PartialEq)]
struct RouteNode {
    cost_m: u32,
    zone_id: u32,
    path: Vec<u32>,
}

impl Ord for RouteNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost_m.cmp(&self.cost_m) // Min-heap
    }
}

impl PartialOrd for RouteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Multi-agent routing solver
pub struct RoutePlanner {
    zones: HashMap<u32, (f64, f64)>, // zone_id -> (lat, lon)
}

impl RoutePlanner {
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
        }
    }

    /// Register a zone location
    pub fn register_zone(&mut self, zone_id: u32, latitude: f64, longitude: f64) {
        self.zones.insert(zone_id, (latitude, longitude));
    }

    /// Haversine distance between two zones in meters
    fn zone_distance(&self, from_zone: u32, to_zone: u32) -> u32 {
        let (lat1, lon1) = self.zones.get(&from_zone).copied().unwrap_or((0.0, 0.0));
        let (lat2, lon2) = self.zones.get(&to_zone).copied().unwrap_or((0.0, 0.0));

        const R: f64 = 6371000.0;
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.to_radians().cos()
                * lat2.to_radians().cos()
                * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        (R * c) as u32
    }

    /// Dijkstra's algorithm: shortest path from start to end
    pub fn shortest_path(&self, start_zone: u32, end_zone: u32) -> Option<Vec<u32>> {
        let mut visited = std::collections::HashSet::new();
        let mut distances: HashMap<u32, u32> = HashMap::new();
        let mut predecessors: HashMap<u32, u32> = HashMap::new();
        let mut queue: BinaryHeap<RouteNode> = BinaryHeap::new();

        distances.insert(start_zone, 0);
        queue.push(RouteNode {
            cost_m: 0,
            zone_id: start_zone,
            path: vec![start_zone],
        });

        while let Some(RouteNode { cost_m, zone_id, path }) = queue.pop() {
            if visited.contains(&zone_id) {
                continue;
            }
            visited.insert(zone_id);

            if zone_id == end_zone {
                return Some(path);
            }

            for neighbor_id in self.zones.keys() {
                if !visited.contains(neighbor_id) {
                    let edge_cost = self.zone_distance(zone_id, *neighbor_id);
                    let new_cost = cost_m + edge_cost;
                    let best_known = distances.get(neighbor_id).copied().unwrap_or(u32::MAX);

                    if new_cost < best_known {
                        distances.insert(*neighbor_id, new_cost);
                        predecessors.insert(*neighbor_id, zone_id);
                        let mut new_path = path.clone();
                        new_path.push(*neighbor_id);
                        queue.push(RouteNode {
                            cost_m: new_cost,
                            zone_id: *neighbor_id,
                            path: new_path,
                        });
                    }
                }
            }
        }

        None
    }

    /// Traveling Salesman Problem approximation (nearest neighbor heuristic)
    pub fn tsp_greedy(&self, start_zone: u32, zones_to_visit: &[u32]) -> Vec<u32> {
        let mut route = vec![start_zone];
        let mut unvisited: std::collections::HashSet<_> = zones_to_visit.iter().copied().collect();

        let mut current = start_zone;
        while !unvisited.is_empty() {
            let next = *unvisited
                .iter()
                .min_by_key(|z| self.zone_distance(current, **z))
                .unwrap();

            route.push(next);
            unvisited.remove(&next);
            current = next;
        }

        route
    }

    /// Calculate total route distance
    pub fn route_distance(&self, route: &[u32]) -> u32 {
        let mut total = 0u32;
        for window in route.windows(2) {
            total += self.zone_distance(window[0], window[1]);
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_distance() {
        let mut planner = RoutePlanner::new();
        planner.register_zone(1, 33.4484, -112.0742); // Phoenix
        planner.register_zone(2, 33.4255, -111.9400); // Tempe

        let dist = planner.zone_distance(1, 2);
        assert!((dist as i32 - 29000).abs() < 3000); // ~29 km
    }

    #[test]
    fn test_shortest_path() {
        let mut planner = RoutePlanner::new();
        for i in 1..=5 {
            planner.register_zone(i, 33.0 + i as f64 * 0.1, -112.0);
        }

        let path = planner.shortest_path(1, 5);
        assert!(path.is_some());
        assert_eq!(path.unwrap()[0], 1);
    }

    #[test]
    fn test_tsp_greedy() {
        let mut planner = RoutePlanner::new();
        planner.register_zone(1, 33.0, -112.0);
        planner.register_zone(2, 33.1, -112.0);
        planner.register_zone(3, 33.2, -112.0);

        let route = planner.tsp_greedy(1, &[2, 3]);
        assert_eq!(route[0], 1);
        assert!(route.len() == 3);
    }

    #[test]
    fn test_route_distance() {
        let mut planner = RoutePlanner::new();
        planner.register_zone(1, 33.0, -112.0);
        planner.register_zone(2, 33.1, -112.0);

        let route = vec![1, 2, 1];
        let dist = planner.route_distance(&route);
        assert!(dist > 0);
    }
}
