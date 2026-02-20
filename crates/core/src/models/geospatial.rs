//! Geospatial data types and coordinate systems

use serde::{Deserialize, Serialize};
use std::fmt;

/// WGS84 geographic coordinate (latitude, longitude)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatLon {
    pub latitude: f64,
    pub longitude: f64,
}

impl LatLon {
    /// Create a new coordinate pair
    pub fn new(latitude: f64, longitude: f64) -> Self {
        assert!(-90.0 <= latitude && latitude <= 90.0, "Invalid latitude");
        assert!(-180.0 <= longitude && longitude <= 180.0, "Invalid longitude");
        Self { latitude, longitude }
    }

    /// Distance to another point in meters (Haversine formula)
    pub fn distance_to(&self, other: &LatLon) -> f64 {
        const R: f64 = 6371000.0; // Earth radius in meters

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        R * c
    }

    /// Bearing to another point in degrees (0-360, 0=North)
    pub fn bearing_to(&self, other: &LatLon) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();

        let y = dlon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
        let bearing = y.atan2(x).to_degrees();

        (bearing + 360.0) % 360.0
    }
}

impl fmt::Display for LatLon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6},{:.6}", self.latitude, self.longitude)
    }
}

/// UTM (Universal Transverse Mercator) coordinate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UTM {
    pub easting: f64,
    pub northing: f64,
    pub zone: u8,
    pub is_north: bool,
}

impl UTM {
    /// Create UTM coordinate (zone 1-60, is_north for Northern hemisphere)
    pub fn new(easting: f64, northing: f64, zone: u8, is_north: bool) -> Self {
        assert!(1 <= zone && zone <= 60, "Invalid UTM zone");
        Self {
            easting,
            northing,
            zone,
            is_north,
        }
    }

    /// Convert to LatLon (approximate, sufficient for 10m accuracy)
    pub fn to_latlon(&self) -> LatLon {
        // Simplified UTM to LatLon (production use geodetic library)
        let lon_origin = (self.zone as f64 - 0.5) * 6.0 - 180.0;
        
        let e = (self.easting - 500000.0) / 0.9996;
        let n = if self.is_north {
            self.northing
        } else {
            self.northing - 10000000.0
        } / 0.9996;

        let lat = n / 6378137.0;
        let lon = lon_origin + (e / (6378137.0 * lat.cos()));

        LatLon::new(lat.to_degrees(), lon)
    }
}

/// Geospatial zone (Sonoran Desert context)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub zone_id: u32,
    pub name: String,
    pub center: LatLon,
    pub area_hectares: f64,
    pub bounds: (LatLon, LatLon), // (southwest, northeast)
    pub metadata: serde_json::Value,
}

impl Zone {
    pub fn new(zone_id: u32, name: String, center: LatLon, area_hectares: f64) -> Self {
        Self {
            zone_id,
            name,
            center,
            area_hectares,
            bounds: (center, center), // Simplification
            metadata: serde_json::json!({}),
        }
    }

    /// Check if point is within zone bounds
    pub fn contains(&self, point: &LatLon) -> bool {
        point.latitude >= self.bounds.0.latitude
            && point.latitude <= self.bounds.1.latitude
            && point.longitude >= self.bounds.0.longitude
            && point.longitude <= self.bounds.1.longitude
    }
}

/// Elevation point (for slope calculation)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElevationPoint {
    pub location: LatLon,
    pub elevation_m: f64,
}

impl ElevationPoint {
    pub fn new(location: LatLon, elevation_m: f64) -> Self {
        Self {
            location,
            elevation_m,
        }
    }

    /// Calculate slope in degrees between two elevation points
    pub fn slope_to(&self, other: &ElevationPoint) -> f64 {
        let distance = self.location.distance_to(&other.location);
        let elevation_diff = (other.elevation_m - self.elevation_m).abs();

        if distance < 1.0 {
            return 0.0; // Avoid division by zero
        }

        (elevation_diff / distance).atan().to_degrees()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latlon_distance() {
        // Phoenix to Tempe (Arizona)
        let phoenix = LatLon::new(33.4484, -112.0742);
        let tempe = LatLon::new(33.4255, -111.9400);
        let dist = phoenix.distance_to(&tempe);
        
        // Should be approximately 29 km
        assert!((dist - 29000.0).abs() < 2000.0);
    }

    #[test]
    fn test_latlon_bearing() {
        let origin = LatLon::new(33.4484, -112.0742);
        let north = LatLon::new(33.5, -112.0742);
        let bearing = origin.bearing_to(&north);
        
        assert!((bearing - 0.0).abs() < 5.0); // ~North
    }

    #[test]
    fn test_zone_contains() {
        let zone = Zone::new(1, "Test Zone".to_string(), LatLon::new(33.5, -112.0), 1000.0);
        let point = LatLon::new(33.5, -112.0);
        
        assert!(zone.contains(&point));
    }

    #[test]
    fn test_slope_calculation() {
        let p1 = ElevationPoint::new(LatLon::new(33.0, -112.0), 100.0);
        let p2 = ElevationPoint::new(LatLon::new(33.001, -112.0), 110.0); // ~111m apart
        let slope = p1.slope_to(&p2);
        
        assert!(slope > 0.0 && slope < 90.0);
    }
}
