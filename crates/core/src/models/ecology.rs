//! Ecological domain models (vegetation, wildlife, recovery metrics)

use serde::{Deserialize, Serialize};

/// Native tree species in Sonoran Desert
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeSpecies {
    Paloverde,      // Parkinsonia (primary native)
    Mesquite,       // Prosopis
    Acacia,         // Acacia greggii
    Saguaro,        // Carnegiea gigantea
    IronWood,       // Olneya tesota
    Creosote,       // Larrea tridentata
}

impl TreeSpecies {
    /// Water requirement (liters/tree/growing season)
    pub fn water_requirement_liters(&self) -> f64 {
        match self {
            Self::Paloverde => 80.0,
            Self::Mesquite => 100.0,
            Self::Acacia => 60.0,
            Self::Saguaro => 40.0,
            Self::IronWood => 120.0,
            Self::Creosote => 30.0,
        }
    }

    /// Carbon sequestration (kg CO2/year/tree)
    pub fn carbon_sequestration_kg_per_year(&self) -> f64 {
        match self {
            Self::Paloverde => 12.0,
            Self::Mesquite => 18.0,
            Self::Acacia => 8.0,
            Self::Saguaro => 5.0,
            Self::IronWood => 22.0,
            Self::Creosote => 4.0,
        }
    }
}

/// Wildlife species inhabiting recovered zones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildlifeSpecies {
    Javelina,
    Roadrunner,
    Gila,            // Gila monster
    CoyoteKid,
    Rattlesnake,
    Hawk,
    Lizard,
}

impl WildlifeSpecies {
    /// Habitat area required (hectares per individual)
    pub fn habitat_hectares_per_individual(&self) -> f64 {
        match self {
            Self::Javelina => 2.0,
            Self::Roadrunner => 0.5,
            Self::Gila => 1.0,
            Self::CoyoteKid => 5.0,
            Self::Rattlesnake => 0.3,
            Self::Hawk => 3.0,
            Self::Lizard => 0.1,
        }
    }

    /// Threat level (0=low, 1=high) from deforestation
    pub fn threat_level(&self) -> f64 {
        match self {
            Self::Javelina => 0.7,
            Self::Roadrunner => 0.5,
            Self::Gila => 0.9,
            Self::CoyoteKid => 0.6,
            Self::Rattlesnake => 0.4,
            Self::Hawk => 0.5,
            Self::Lizard => 0.3,
        }
    }
}

/// Vegetation canopy cover classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VegetationClass {
    Bare,       // 0-10% cover
    Sparse,     // 10-30% cover
    Moderate,   // 30-60% cover
    Dense,      // 60-100% cover
}

impl VegetationClass {
    /// Convert from tree density (trees/hectare)
    pub fn from_tree_density(trees_per_ha: f64) -> Self {
        match trees_per_ha {
            t if t < 50.0 => Self::Bare,
            t if t < 150.0 => Self::Sparse,
            t if t < 300.0 => Self::Moderate,
            _ => Self::Dense,
        }
    }

    /// Estimated canopy cover percentage
    pub fn cover_percent(&self) -> f64 {
        match self {
            Self::Bare => 5.0,
            Self::Sparse => 20.0,
            Self::Moderate => 45.0,
            Self::Dense => 80.0,
        }
    }
}

/// Soil health indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoilHealth {
    pub organic_matter_percent: f64,  // 0-5%
    pub moisture_content_percent: f64, // 0-30%
    pub compaction_depth_cm: f64,     // 0-30 cm
    pub ph: f64,                       // 4-9
}

impl SoilHealth {
    /// Calculate composite soil health score (0.0-1.0)
    pub fn score(&self) -> f64 {
        let organic = (self.organic_matter_percent / 5.0).min(1.0);
        let moisture = (self.moisture_content_percent / 20.0).min(1.0);
        let compaction = 1.0 - (self.compaction_depth_cm / 30.0).min(1.0);
        let ph = if (self.ph - 7.0).abs() < 1.0 { 1.0 } else { 0.7 };

        (organic + moisture + compaction + ph) / 4.0
    }
}

impl Default for SoilHealth {
    fn default() -> Self {
        Self {
            organic_matter_percent: 1.0,
            moisture_content_percent: 5.0,
            compaction_depth_cm: 15.0,
            ph: 7.5,
        }
    }
}

/// Ecological recovery stage
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStage {
    Bare,           // Stage 0: Post-deforestation
    Establishing,   // Stage 1: Initial restoration (0-2 years)
    Developing,     // Stage 2: Growth phase (2-5 years)
    Maturing,       // Stage 3: Ecosystem stabilization (5-10 years)
    Recovered,      // Stage 4: Full recovery (10+ years)
}

impl RecoveryStage {
    /// Numeric progression (0.0-1.0)
    pub fn progress(&self) -> f64 {
        match self {
            Self::Bare => 0.0,
            Self::Establishing => 0.2,
            Self::Developing => 0.5,
            Self::Maturing => 0.8,
            Self::Recovered => 1.0,
        }
    }

    /// Time in years to reach this stage from Bare
    pub fn years_from_bare(&self) -> f64 {
        match self {
            Self::Bare => 0.0,
            Self::Establishing => 1.0,
            Self::Developing => 3.5,
            Self::Maturing => 7.5,
            Self::Recovered => 12.0,
        }
    }
}

/// Ecological zone snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologicalZone {
    pub zone_id: u32,
    pub tree_species: Vec<TreeSpecies>,
    pub trees_per_hectare: f64,
    pub vegetation_class: VegetationClass,
    pub soil_health: SoilHealth,
    pub wildlife_population: std::collections::HashMap<String, u32>,
    pub recovery_stage: RecoveryStage,
    pub water_deficit_mm: f64,
    pub invasive_species_percent: f64,
}

impl EcologicalZone {
    pub fn new(zone_id: u32) -> Self {
        Self {
            zone_id,
            tree_species: vec![],
            trees_per_hectare: 0.0,
            vegetation_class: VegetationClass::Bare,
            soil_health: SoilHealth::default(),
            wildlife_population: std::collections::HashMap::new(),
            recovery_stage: RecoveryStage::Bare,
            water_deficit_mm: 50.0,
            invasive_species_percent: 0.0,
        }
    }

    /// Ecosystem resilience (0.0-1.0)
    pub fn resilience(&self) -> f64 {
        let recovery_factor = self.recovery_stage.progress();
        let soil_factor = self.soil_health.score();
        let wildlife_factor = if self.wildlife_population.is_empty() {
            0.2
        } else {
            0.8
        };
        let invasive_factor = 1.0 - (self.invasive_species_percent / 100.0);

        (recovery_factor * 0.3 + soil_factor * 0.25 + wildlife_factor * 0.25 + invasive_factor * 0.2)
            .min(1.0)
    }

    /// Total carbon potential (kg CO2/hectare/year)
    pub fn carbon_potential(&self) -> f64 {
        if self.tree_species.is_empty() {
            return 0.0;
        }

        let avg_carbon = self.tree_species
            .iter()
            .map(|s| s.carbon_sequestration_kg_per_year())
            .sum::<f64>()
            / self.tree_species.len() as f64;

        avg_carbon * self.trees_per_hectare
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_species_water() {
        let water = TreeSpecies::Mesquite.water_requirement_liters();
        assert!(water > 50.0 && water < 150.0);
    }

    #[test]
    fn test_vegetation_class_from_density() {
        let vc = VegetationClass::from_tree_density(200.0);
        assert_eq!(vc, VegetationClass::Moderate);
    }

    #[test]
    fn test_soil_health_score() {
        let soil = SoilHealth {
            organic_matter_percent: 3.0,
            moisture_content_percent: 15.0,
            compaction_depth_cm: 10.0,
            ph: 7.2,
        };
        let score = soil.score();
        assert!(0.5 < score && score <= 1.0);
    }

    #[test]
    fn test_recovery_stage_progression() {
        assert_eq!(RecoveryStage::Bare.progress(), 0.0);
        assert_eq!(RecoveryStage::Recovered.progress(), 1.0);
    }

    #[test]
    fn test_ecological_zone_resilience() {
        let mut zone = EcologicalZone::new(1);
        zone.recovery_stage = RecoveryStage::Developing;
        zone.soil_health.organic_matter_percent = 2.5;
        
        let resilience = zone.resilience();
        assert!(0.0 <= resilience && resilience <= 1.0);
    }
}
