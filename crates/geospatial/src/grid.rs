//! Regular spatial grid for efficient zone management

use cybersomething_core::models::LatLon;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Regular geographic grid cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub cell_id: u32,
    pub row: u32,
    pub col: u32,
    pub center: LatLon,
    pub bounds: (LatLon, LatLon), // (sw, ne)
    pub data: HashMap<String, f64>,
}

impl GridCell {
    pub fn new(cell_id: u32, row: u32, col: u32, center: LatLon) -> Self {
        Self {
            cell_id,
            row,
            col,
            center,
            bounds: (center, center), // Simplified
            data: HashMap::new(),
        }
    }

    /// Check if point is within cell bounds
    pub fn contains(&self, point: &LatLon) -> bool {
        point.latitude >= self.bounds.0.latitude
            && point.latitude <= self.bounds.1.latitude
            && point.longitude >= self.bounds.0.longitude
            && point.longitude <= self.bounds.1.longitude
    }

    /// Set raster value
    pub fn set_value(&mut self, key: &str, value: f64) {
        self.data.insert(key.to_string(), value);
    }

    /// Get raster value
    pub fn get_value(&self, key: &str) -> Option<f64> {
        self.data.get(key).copied()
    }
}

/// Regular square grid covering a region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialGrid {
    pub grid_id: u32,
    pub rows: u32,
    pub cols: u32,
    pub cell_size_km: f64,
    pub cells: HashMap<u32, GridCell>,
}

impl SpatialGrid {
    /// Create grid with specified dimensions and cell size
    pub fn new(grid_id: u32, rows: u32, cols: u32, cell_size_km: f64) -> Self {
        Self {
            grid_id,
            rows,
            cols,
            cell_size_km,
            cells: HashMap::new(),
        }
    }

    /// Initialize grid with cells
    pub fn initialize(&mut self, origin_lat: f64, origin_lon: f64) {
        let lat_step = self.cell_size_km / 111.0; // degrees per km at equator
        let lon_step = self.cell_size_km / 111.0;

        let mut cell_id = 0u32;
        for row in 0..self.rows {
            for col in 0..self.cols {
                let lat = origin_lat + row as f64 * lat_step;
                let lon = origin_lon + col as f64 * lon_step;
                let center = LatLon::new(lat, lon);

                let cell = GridCell::new(cell_id, row, col, center);
                self.cells.insert(cell_id, cell);
                cell_id += 1;
            }
        }
    }

    /// Get cell containing point
    pub fn get_cell_at(&self, point: &LatLon) -> Option<&GridCell> {
        self.cells.values().find(|c| c.contains(point))
    }

    /// Get cell by row and column
    pub fn get_cell(&self, row: u32, col: u32) -> Option<&GridCell> {
        let cell_id = row * self.cols + col;
        self.cells.get(&cell_id)
    }

    /// Get cell mutably
    pub fn get_cell_mut(&mut self, row: u32, col: u32) -> Option<&mut GridCell> {
        let cell_id = row * self.cols + col;
        self.cells.get_mut(&cell_id)
    }

    /// Get neighbors of a cell (4-connectivity)
    pub fn get_neighbors(&self, row: u32, col: u32) -> Vec<&GridCell> {
        let mut neighbors = Vec::new();

        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in directions.iter() {
            let new_row = (row as i32 + dr) as u32;
            let new_col = (col as i32 + dc) as u32;

            if new_row < self.rows && new_col < self.cols {
                if let Some(cell) = self.get_cell(new_row, new_col) {
                    neighbors.push(cell);
                }
            }
        }

        neighbors
    }

    /// Aggregate value across all cells
    pub fn aggregate(&self, key: &str) -> f64 {
        self.cells
            .values()
            .filter_map(|c| c.get_value(key))
            .sum()
    }

    /// Average value across cells
    pub fn average(&self, key: &str) -> f64 {
        let values: Vec<f64> = self
            .cells
            .values()
            .filter_map(|c| c.get_value(key))
            .collect();

        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    /// Gaussian blur on grid (smoothing)
    pub fn blur(&mut self, key: &str, sigma: f64) {
        let mut blurred = HashMap::new();

        for cell in self.cells.values() {
            let neighbors = self.get_neighbors(cell.row, cell.col);
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            if let Some(center_val) = cell.get_value(key) {
                weighted_sum += center_val;
                weight_sum += 1.0;
            }

            for neighbor in neighbors {
                if let Some(neighbor_val) = neighbor.get_value(key) {
                    let weight = (-1.0 / (2.0 * sigma * sigma)).exp();
                    weighted_sum += neighbor_val * weight;
                    weight_sum += weight;
                }
            }

            if weight_sum > 0.0 {
                blurred.insert(cell.cell_id, weighted_sum / weight_sum);
            }
        }

        for (cell_id, value) in blurred {
            if let Some(cell) = self.cells.get_mut(&cell_id) {
                cell.set_value(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = SpatialGrid::new(1, 10, 10, 1.0);
        assert_eq!(grid.rows, 10);
        assert_eq!(grid.cols, 10);
    }

    #[test]
    fn test_grid_initialization() {
        let mut grid = SpatialGrid::new(1, 5, 5, 1.0);
        grid.initialize(33.0, -112.0);

        assert_eq!(grid.cells.len(), 25);
    }

    #[test]
    fn test_cell_lookup() {
        let mut grid = SpatialGrid::new(1, 3, 3, 1.0);
        grid.initialize(33.0, -112.0);

        let cell = grid.get_cell(0, 0);
        assert!(cell.is_some());
    }

    #[test]
    fn test_neighbors() {
        let mut grid = SpatialGrid::new(1, 5, 5, 1.0);
        grid.initialize(33.0, -112.0);

        let neighbors = grid.get_neighbors(2, 2);
        assert_eq!(neighbors.len(), 4); // Center cell has 4 neighbors
    }

    #[test]
    fn test_grid_aggregate() {
        let mut grid = SpatialGrid::new(1, 3, 3, 1.0);
        grid.initialize(33.0, -112.0);

        for cell in grid.cells.values_mut() {
            cell.set_value("value", 10.0);
        }

        let total = grid.aggregate("value");
        assert_eq!(total, 90.0); // 9 cells * 10.0
    }
}
