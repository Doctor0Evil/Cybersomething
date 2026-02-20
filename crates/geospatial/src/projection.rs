//! Coordinate system transformations (WGS84, UTM, local projections)

use cybersomething_core::models::{LatLon, UTM};
use serde::{Deserialize, Serialize};

/// Projection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionType {
    WGS84,
    UTM,
    LocalCartesian,
}

/// Coordinate transformer
pub struct CoordinateTransformer {
    source_proj: ProjectionType,
    target_proj: ProjectionType,
    utm_zone: u8,
}

impl CoordinateTransformer {
    pub fn new(source_proj: ProjectionType, target_proj: ProjectionType) -> Self {
        Self {
            source_proj,
            target_proj,
            utm_zone: 11, // Arizona
        }
    }

    /// Transform WGS84 to UTM11N (Sonoran Desert)
    pub fn latlon_to_utm(&self, latlon: &LatLon) -> UTM {
        let lon_origin = (self.utm_zone as f64 - 0.5) * 6.0 - 180.0;
        let dlon = (latlon.longitude - lon_origin).to_radians();
        let lat = latlon.latitude.to_radians();

        const K0: f64 = 0.9996;
        const E2: f64 = 0.00669438; // WGS84 eccentricity squared
        const A: f64 = 6378137.0;

        let n = A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
        let t = lat.tan().powi(2);
        let c = lat.cos().powi(2) * E2 / (1.0 - E2);

        let easting = K0
            * n
            * (dlon
                + dlon.powi(3) / 6.0 * lat.cos().powi(2) * (1.0 - t + c)
                + dlon.powi(5) / 120.0
                    * lat.cos().powi(4)
                    * (5.0 - 18.0 * t + t.powi(2) + 72.0 * c - 58.0 * E2))
            + 500000.0;

        // Meridian arc
        let a = A
            / (1.0 + ((1.0 - E2) / (1.0 - E2 * lat.sin().powi(2))).sqrt())
            * (1.0 - E2 / 4.0 - 3.0 * E2.powi(2) / 64.0 - 5.0 * E2.powi(3) / 256.0);
        let alpha1 = 3.0 / 2.0 * E2 - 27.0 / 32.0 * E2.powi(3);
        let alpha2 = 21.0 / 16.0 * E2.powi(2) - 55.0 / 32.0 * E2.powi(4);

        let m = a * (lat - alpha1 * (2.0 * lat).sin() + alpha2 * (4.0 * lat).sin());
        let northing = K0 * m;

        UTM::new(
            easting,
            if latlon.latitude >= 0.0 { northing } else { northing + 10000000.0 },
            self.utm_zone,
            latlon.latitude >= 0.0,
        )
    }

    /// Transform coordinate based on projection types
    pub fn transform(&self, source_coord: &str) -> Option<String> {
        match (self.source_proj, self.target_proj) {
            (ProjectionType::WGS84, ProjectionType::UTM) => {
                let parts: Vec<&str> = source_coord.split(',').collect();
                if parts.len() == 2 {
                    let lat = parts[0].trim().parse::<f64>().ok()?;
                    let lon = parts[1].trim().parse::<f64>().ok()?;
                    let latlon = LatLon::new(lat, lon);
                    let utm = self.latlon_to_utm(&latlon);

                    Some(format!("{} {}", utm.easting as i32, utm.northing as i32))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Local Cartesian coordinate system (for local planning)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LocalCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl LocalCoordinate {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Distance to another local coordinate
    pub fn distance_to(&self, other: &LocalCoordinate) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Geoid height model (for elevation corrections)
pub struct GeoidHeight;

impl GeoidHeight {
    /// Approximate geoid height for Sonoran Desert (simplified)
    pub fn at_location(lat: f64, lon: f64) -> f64 {
        // Simplified: returns approximate geoid undulation for Arizona
        // In reality, would use full model or gridded data
        -22.0 + lat * 0.1 + lon.abs() * 0.05
    }

    /// Ellipsoidal to orthometric height conversion
    pub fn ellipsoidal_to_orthometric(ellipsoidal_height: f64, lat: f64, lon: f64) -> f64 {
        ellipsoidal_height - Self::at_location(lat, lon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_creation() {
        let transformer = CoordinateTransformer::new(ProjectionType::WGS84, ProjectionType::UTM);
        assert_eq!(transformer.source_proj, ProjectionType::WGS84);
    }

    #[test]
    fn test_latlon_to_utm() {
        let transformer = CoordinateTransformer::new(ProjectionType::WGS84, ProjectionType::UTM);

        let phoenix = LatLon::new(33.4484, -112.0742);
        let utm = transformer.latlon_to_utm(&phoenix);

        // Phoenix should be in UTM zone 11N
        assert_eq!(utm.zone, 11);
        assert!(utm.is_north);

        // Easting should be around 400,000-700,000m
        assert!(utm.easting > 200000.0 && utm.easting < 800000.0);
    }

    #[test]
    fn test_local_coordinate_distance() {
        let c1 = LocalCoordinate::new(0.0, 0.0, 0.0);
        let c2 = LocalCoordinate::new(3.0, 4.0, 0.0);

        let dist = c1.distance_to(&c2);
        assert_eq!(dist, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_geoid_height() {
        let geoid = GeoidHeight::at_location(33.0, -112.0);
        assert!(geoid < 0.0); // Below ellipsoid in most of world
    }
}
