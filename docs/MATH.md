# Cybersomething Core Mathematics

## Risk Index (Pi)

**Pi = α·Vi + β·Gi + γ·Si**

- Vi: Vegetation density (trees/hectare), normalized to [0,1]
- Gi: Invasive grass coverage (%), normalized to [0,1]  
- Si: Slope steepness (degrees), normalized to [0,1]
- α = 0.35, β = 0.45, γ = 0.20 (Sonoran calibration)

**Defensible Zones:**
- Pi < 0.33 (Low): 0-1m bare, 1-10m 10cm grass, 10m+ 20cm grass
- 0.33 ≤ Pi < 0.67 (Medium): 0-10m 10cm, 10m+ 20cm
- Pi ≥ 0.67 (High): Full 0-30m defensible space

## Water-Bottle Delivery

**Zone Priority Score:**
Pz = (Deficit_mm / 100) + (Wildlife_Count / 100) - (Recovery_Stage × 2.0) - (Distance_km / 10)

- Maximize water delivery to highest Pz zones first
- Minimize transport distance and energy cost

## Swarm Energy Budget

**Drone mission:**
- Flight: 0.5 J/meter
- Payload delivery: 1000 J/mission
- Total energy: E_total = 0.5·d + 1000 + 500·(time_airborne_minutes)

**Nanobot mission:**
- Soil remediation: 0.1 J/meter
- Nutrient injection: 50 J/injection
- Recharge interval: 4 hours (solar/RF harvesting)
