//! Vector geometry types (points, lines, polygons)

use cybersomething_core::models::LatLon;
use serde::{Deserialize, Serialize};

/// Vector geometry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    Point(LatLon),
    LineString(Vec<LatLon>),
    Polygon(Vec<LatLon>), // Exterior ring
    MultiPolygon(Vec<Vec<LatLon>>),
}

impl Geometry {
    /// Get bounding box
    pub fn bounds(&self) -> Option<(LatLon, LatLon)> {
        match self {
            Self::Point(p) => Some((*p, *p)),
            Self::LineString(coords) | Self::Polygon(coords) => {
                let lats: Vec<f64> = coords.iter().map(|c| c.latitude).collect();
                let lons: Vec<f64> = coords.iter().map(|c| c.longitude).collect();

                if lats.is_empty() {
                    return None;
                }

                let min_lat = lats.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_lat = lats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let min_lon = lons.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_lon = lons.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                Some((
                    LatLon::new(min_lat, min_lon),
                    LatLon::new(max_lat, max_lon),
                ))
            }
            Self::MultiPolygon(polys) => {
                let all_coords: Vec<LatLon> = polys.iter().flat_map(|p| p.clone()).collect();
                if all_coords.is_empty() {
                    return None;
                }

                let lats: Vec<f64> = all_coords.iter().map(|c| c.latitude).collect();
                let lons: Vec<f64> = all_coords.iter().map(|c| c.longitude).collect();

                let min_lat = lats.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_lat = lats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let min_lon = lons.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_lon = lons.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                Some((
                    LatLon::new(min_lat, min_lon),
                    LatLon::new(max_lat, max_lon),
                ))
            }
        }
    }

    /// Calculate length (for LineString in meters)
    pub fn length_m(&self) -> f64 {
        match self {
            Self::LineString(coords) => {
                coords
                    .windows(2)
                    .map(|w| w[0].distance_to(&w[1]))
                    .sum()
            }
            _ => 0.0,
        }
    }

    /// Calculate area (for Polygon in square meters, simplified)
    pub fn area_m2(&self) -> f64 {
        match self {
            Self::Polygon(coords) => {
                if coords.len() < 3 {
                    return 0.0;
                }

                // Shoelace formula (simplified for small areas)
                let mut area = 0.0;
                for i in 0..coords.len() {
                    let j = (i + 1) % coords.len();
                    area += coords[i].longitude * coords[j].latitude;
                    area -= coords[j].longitude * coords[i].latitude;
                }

                (area.abs() / 2.0) * 111000.0 * 111000.0 // Approximate m2
            }
            _ => 0.0,
        }
    }

    /// Point-in-polygon test (ray casting algorithm)
    pub fn contains_point(&self, point: &LatLon) -> bool {
        match self {
            Self::Polygon(coords) => {
                if coords.len() < 3 {
                    return false;
                }

                let mut inside = false;
                let mut j = coords.len() - 1;

                for i in 0..coords.len() {
                    if (coords[i].latitude > point.latitude) != (coords[j].latitude > point.latitude)
                        && point.longitude
                            < (coords[j].longitude - coords[i].longitude)
                                * (point.latitude - coords[i].latitude)
                                / (coords[j].latitude - coords[i].latitude)
                                + coords[i].longitude
                    {
                        inside = !inside;
                    }
                    j = i;
                }

                inside
            }
            Self::Point(p) => point == p,
            _ => false,
        }
    }
}

/// Vector feature (geometry + attributes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub feature_id: u32,
    pub geometry: Geometry,
    pub properties: std::collections::HashMap<String, String>,
}

impl Feature {
    pub fn new(feature_id: u32, geometry: Geometry) -> Self {
        Self {
            feature_id,
            geometry,
            properties: std::collections::HashMap::new(),
        }
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

/// Feature collection (layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCollection {
    pub collection_id: u32,
    pub name: String,
    pub features: Vec<Feature>,
}

impl FeatureCollection {
    pub fn new(collection_id: u32, name: String) -> Self {
        Self {
            collection_id,
            name,
            features: Vec::new(),
        }
    }

    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Find features intersecting bounds
    pub fn query_bounds(&self, bounds: (LatLon, LatLon)) -> Vec<&Feature> {
        self.features
            .iter()
            .filter(|f| {
                if let Some((sw, ne)) = f.geometry.bounds() {
                    // Check if bounds intersect
                    !(ne.latitude < bounds.0.latitude
                        || sw.latitude > bounds.1.latitude
                        || ne.longitude < bounds.0.longitude
                        || sw.longitude > bounds.1.longitude)
                } else {
                    false
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_geometry() {
        let pt = Geometry::Point(LatLon::new(33.0, -112.0));
        let bounds = pt.bounds();
        assert!(bounds.is_some());
    }

    #[test]
    fn test_linestring_length() {
        let coords = vec![
            LatLon::new(33.0, -112.0),
            LatLon::new(33.1, -112.0),
        ];
        let line = Geometry::LineString(coords);
        let length = line.length_m();

        assert!(length > 10000.0); // Should be ~11 km
    }

    #[test]
    fn test_polygon_area() {
        let coords = vec![
            LatLon::new(33.0, -112.0),
            LatLon::new(33.1, -112.0),
            LatLon::new(33.1, -111.9),
            LatLon::new(33.0, -111.9),
        ];
        let poly = Geometry::Polygon(coords);
        let area = poly.area_m2();

        assert!(area > 0.0);
    }

    #[test]
    fn test_point_in_polygon() {
        let coords = vec![
            LatLon::new(33.0, -112.0),
            LatLon::new(33.1, -112.0),
            LatLon::new(33.1, -111.9),
            LatLon::new(33.0, -111.9),
        ];
        let poly = Geometry::Polygon(coords);

        let inside = LatLon::new(33.05, -111.95);
        assert!(poly.contains_point(&inside));
    }

    #[test]
    fn test_feature_creation() {
        let geom = Geometry::Point(LatLon::new(33.0, -112.0));
        let mut feature = Feature::new(1, geom);

        feature.set_property("name".to_string(), "Test Point".to_string());
        assert_eq!(feature.get_property("name"), Some(&"Test Point".to_string()));
    }

    #[test]
    fn test_collection_query() {
        let mut collection = FeatureCollection::new(1, "Test".to_string());

        let geom = Geometry::Point(LatLon::new(33.05, -111.95));
        let feature = Feature::new(1, geom);
        collection.add_feature(feature);

        let bounds = (LatLon::new(33.0, -112.0), LatLon::new(33.1, -111.9));
        let results = collection.query_bounds(bounds);

        assert_eq!(results.len(), 1);
    }
}
