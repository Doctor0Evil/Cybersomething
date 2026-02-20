//! Sonoran Desert Parcel Risk Index
//! Pi = α·Vi + β·Gi + γ·Si
//! Where:
//!   Vi = Vegetation density (trees/ha)
//!   Gi = Invasive grass cover (%)
//!   Si = Slope steepness (degrees)

use approx::abs_diff_eq;

/// Risk coefficients calibrated for Sonoran Desert WUI
pub struct RiskWeights {
    pub alpha: f64,   // Vegetation density weight
    pub beta: f64,    // Invasive grass weight
    pub gamma: f64,   // Slope weight
}

impl Default for RiskWeights {
    fn default() -> Self {
        Self {
            alpha: 0.35,   // Vegetation moderate risk
            beta: 0.45,    // Invasive grass = HIGH risk
            gamma: 0.20,   // Slope amplification
        }
    }
}

/// Parcel risk index calculation
pub struct RiskCalculator {
    weights: RiskWeights,
}

impl RiskCalculator {
    pub fn new(weights: RiskWeights) -> Self {
        Self { weights }
    }

    /// Compute risk index for a parcel (0.0 = safe, 1.0 = critical)
    pub fn compute_risk(&self, vi: f64, gi: f64, si: f64) -> f64 {
        // Normalize inputs to [0, 1]
        let vi_norm = (vi / 1000.0).min(1.0);  // Max 1000 trees/ha
        let gi_norm = gi / 100.0;               // Already percentage
        let si_norm = (si / 60.0).min(1.0);    // Max 60° slope

        // Weighted sum
        let pi = self.weights.alpha * vi_norm
            + self.weights.beta * gi_norm
            + self.weights.gamma * si_norm;

        pi.min(1.0)  // Clamp to [0, 1]
    }

    /// Defensible zone recommendation (meters) based on risk
    pub fn defensible_zone(&self, risk_index: f64) -> (u32, u32, u32) {
        match risk_index {
            r if r < 0.33 => (0, 100, 20),    // Low: 0-1m (0cm), 1-10m (10cm), 10m+ (20cm)
            r if r < 0.67 => (100, 200, 20),  // Medium: 0-10m (10cm), 10m+ (20cm)
            _ => (200, 300, 30),              // High: full 0-30m defensible space
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_calculation() {
        let calc = RiskCalculator::new(RiskWeights::default());
        let risk = calc.compute_risk(500.0, 80.0, 15.0);  // Med veg, high grass, mod slope
        assert!(0.0 <= risk && risk <= 1.0);
    }

    #[test]
    fn test_defensible_zone_low_risk() {
        let calc = RiskCalculator::new(RiskWeights::default());
        let (inner, mid, outer) = calc.defensible_zone(0.2);
        assert_eq!(inner, 0);
    }
}
