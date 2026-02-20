//! Raster data handling for satellite and UAV imagery

use cybersomething_core::models::LatLon;
use ndarray::{Array2, ArrayView2};
use serde::{Deserialize, Serialize};

/// Single raster band (e.g., NDVI, elevation, temperature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterBand {
    pub band_id: u32,
    pub band_name: String,
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize,
    pub no_data_value: f32,
    pub min_value: f32,
    pub max_value: f32,
}

impl RasterBand {
    pub fn new(band_id: u32, band_name: String, rows: usize, cols: usize) -> Self {
        Self {
            band_id,
            band_name,
            data: vec![0.0; rows * cols],
            rows,
            cols,
            no_data_value: -9999.0,
            min_value: f32::MAX,
            max_value: f32::MIN,
        }
    }

    /// Set pixel value
    pub fn set_pixel(&mut self, row: usize, col: usize, value: f32) {
        if row < self.rows && col < self.cols {
            let idx = row * self.cols + col;
            self.data[idx] = value;

            self.min_value = self.min_value.min(value);
            self.max_value = self.max_value.max(value);
        }
    }

    /// Get pixel value
    pub fn get_pixel(&self, row: usize, col: usize) -> Option<f32> {
        if row < self.rows && col < self.cols {
            let idx = row * self.cols + col;
            Some(self.data[idx])
        } else {
            None
        }
    }

    /// Normalize band to [0, 1]
    pub fn normalize(&self) -> Vec<f32> {
        let range = self.max_value - self.min_value;
        if range < 0.0001 {
            return vec![0.5; self.data.len()];
        }

        self.data
            .iter()
            .map(|&v| (v - self.min_value) / range)
            .collect()
    }

    /// Compute statistics
    pub fn statistics(&self) -> RasterStats {
        let valid_values: Vec<f32> = self
            .data
            .iter()
            .filter(|&&v| v != self.no_data_value)
            .copied()
            .collect();

        if valid_values.is_empty() {
            return RasterStats::default();
        }

        let mean = valid_values.iter().sum::<f32>() / valid_values.len() as f32;
        let variance: f32 = valid_values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f32>()
            / valid_values.len() as f32;

        RasterStats {
            count: valid_values.len() as u32,
            min: *valid_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            max: *valid_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            mean,
            std_dev: variance.sqrt(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RasterStats {
    pub count: u32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub std_dev: f32,
}

/// Multi-band raster dataset (e.g., satellite image)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterDataset {
    pub dataset_id: u32,
    pub bands: Vec<RasterBand>,
    pub extent: (LatLon, LatLon), // (sw, ne)
    pub crs: CoordinateSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateSystem {
    WGS84,
    UTM11N,
    LocalProjection,
}

impl RasterDataset {
    pub fn new(dataset_id: u32, extent: (LatLon, LatLon)) -> Self {
        Self {
            dataset_id,
            bands: Vec::new(),
            extent,
            crs: CoordinateSystem::WGS84,
        }
    }

    /// Add a band to the dataset
    pub fn add_band(&mut self, band: RasterBand) {
        self.bands.push(band);
    }

    /// Get band by name
    pub fn get_band(&self, band_name: &str) -> Option<&RasterBand> {
        self.bands.iter().find(|b| b.band_name == band_name)
    }

    /// Compute NDVI (Normalized Difference Vegetation Index)
    /// NDVI = (NIR - Red) / (NIR + Red)
    pub fn compute_ndvi(&self) -> Option<RasterBand> {
        let nir = self.get_band("NIR")?;
        let red = self.get_band("Red")?;

        let mut ndvi = RasterBand::new(999, "NDVI".to_string(), nir.rows, nir.cols);

        for row in 0..nir.rows {
            for col in 0..nir.cols {
                if let (Some(nir_val), Some(red_val)) = (nir.get_pixel(row, col), red.get_pixel(row, col)) {
                    if nir_val + red_val > 0.001 {
                        let value = (nir_val - red_val) / (nir_val + red_val);
                        ndvi.set_pixel(row, col, value);
                    }
                }
            }
        }

        Some(ndvi)
    }

    /// Classify pixels by value thresholds
    pub fn classify(&self, band_name: &str, thresholds: &[f32]) -> Option<Vec<u8>> {
        let band = self.get_band(band_name)?;

        let classification = band
            .data
            .iter()
            .map(|&v| {
                for (idx, &threshold) in thresholds.iter().enumerate() {
                    if v < threshold {
                        return idx as u8;
                    }
                }
                thresholds.len() as u8
            })
            .collect();

        Some(classification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raster_band_creation() {
        let band = RasterBand::new(1, "Red".to_string(), 10, 10);
        assert_eq!(band.rows, 10);
        assert_eq!(band.data.len(), 100);
    }

    #[test]
    fn test_pixel_operations() {
        let mut band = RasterBand::new(1, "Red".to_string(), 5, 5);
        band.set_pixel(2, 2, 50.0);

        assert_eq!(band.get_pixel(2, 2), Some(50.0));
    }

    #[test]
    fn test_raster_normalization() {
        let mut band = RasterBand::new(1, "Red".to_string(), 2, 2);
        band.set_pixel(0, 0, 0.0);
        band.set_pixel(1, 1, 100.0);

        let normalized = band.normalize();
        assert!(normalized[0] <= 1.0 && normalized[0] >= 0.0);
    }

    #[test]
    fn test_dataset_creation() {
        let sw = LatLon::new(33.0, -112.0);
        let ne = LatLon::new(33.5, -111.5);
        let dataset = RasterDataset::new(1, (sw, ne));

        assert_eq!(dataset.bands.len(), 0);
    }
}
