//! Water delivery routing for ecological recovery
//! "Water-bottle method": Calculate water bottle deployment to deforested zones

use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct HydroZone {
    pub zone_id: u32,
    pub center_lat: f64,
    pub center_lon: f64,
    pub deficit_mm: f64,           // Water needed (mm)
    pub native_species_count: u32, // Wildlife count
    pub recovery_stage: f64,       // 0.0 = bare, 1.0 = recovered
}

#[derive(Debug, Clone, Copy)]
pub struct WaterBottle {
    pub id: u32,
    pub capacity_liters: f64,
    pub source_lat: f64,
    pub source_lon: f64,
}

#[derive(Clone, Eq, PartialEq)]
struct RouteNode {
    zone_id: u32,
    cost: u64,  // distance in meters, quantized
}

impl Ord for RouteNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)  // Min-heap
    }
}

impl PartialOrd for RouteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HydroRouter;

impl HydroRouter {
    /// Haversine distance (m) between two points
    pub fn distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371000.0; // Earth radius (m)
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.to_radians().cos()
                * lat2.to_radians().cos()
                * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        R * c
    }

    /// Greedy water bottle routing: visit highest priority zones first
    pub fn route_bottles(
        bottles: &[WaterBottle],
        zones: &[HydroZone],
    ) -> Vec<(u32, u32, f64)> {
        // (bottle_id, zone_id, distance_m)
        let mut routes = Vec::new();

        for bottle in bottles {
            // Find zone with highest recovery need
            let mut best_zone: Option<(u32, f64)> = None;
            let mut best_priority = f64::NEG_INFINITY;

            for zone in zones {
                let dist = Self::distance(
                    bottle.source_lat,
                    bottle.source_lon,
                    zone.center_lat,
                    zone.center_lon,
                );

                // Priority = water need + ecosystem urgency - distance penalty
                let priority = (zone.deficit_mm as f64 / 100.0)
                    + (zone.native_species_count as f64 / 100.0)
                    - (zone.recovery_stage * 2.0)
                    - (dist / 10000.0);

                if priority > best_priority {
                    best_priority = priority;
                    best_zone = Some((zone.zone_id, dist));
                }
            }

            if let Some((zone_id, dist)) = best_zone {
                routes.push((bottle.id, zone_id, dist));
            }
        }

        routes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine() {
        // Phoenix to Tempe ≈ 29 km
        let dist = HydroRouter::distance(33.4484, -112.0742, 33.4255, -111.9400);
        assert!((dist - 29000.0).abs() < 5000.0);
    }

    #[test]
    fn test_bottle_routing() {
        let bottles = vec![WaterBottle {
            id: 1,
            capacity_liters: 1000.0,
            source_lat: 33.4484,
            source_lon: -112.0742,
        }];

        let zones = vec![
            HydroZone {
                zone_id: 101,
                center_lat: 33.4484,
                center_lon: -112.0742,
                deficit_mm: 100.0,
                native_species_count: 50,
                recovery_stage: 0.2,
            },
            HydroZone {
                zone_id: 102,
                center_lat: 33.4255,
                center_lon: -111.9400,
                deficit_mm: 150.0,
                native_species_count: 100,
                recovery_stage: 0.1,
            },
        ];

        let routes = HydroRouter::route_bottles(&bottles, &zones);
        assert_eq!(routes.len(), 1);
    }
}
