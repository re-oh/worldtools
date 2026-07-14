use worldtools_world::TerrainSettings;

use crate::{
    AtlasGrid,
    layers::{Biome, SoilKind},
    stages::{
        climate::ClimateState, geology::GeologyState, hydrology::HydrologyState, math::smoothstep,
        tectonics::TectonicState,
    },
};

#[derive(Debug)]
pub(crate) struct SoilState {
    pub(crate) kind: Vec<u8>,
    pub(crate) depth_m: Vec<f32>,
    pub(crate) fertility: Vec<f32>,
    pub(crate) clay_fraction: Vec<f32>,
    pub(crate) organic_fraction: Vec<f32>,
    pub(crate) drainage: Vec<f32>,
}

#[derive(Debug)]
pub(crate) struct VegetationState {
    pub(crate) biome: Vec<u8>,
    pub(crate) canopy_fraction: Vec<f32>,
    pub(crate) grass_fraction: Vec<f32>,
    pub(crate) biomass: Vec<f32>,
    pub(crate) fire_frequency: Vec<f32>,
}

#[allow(clippy::too_many_lines)] // Per-cell outputs remain aligned in one pass.
pub(crate) fn simulate(
    grid: AtlasGrid,
    terrain: TerrainSettings,
    tectonics: &TectonicState,
    climate: &ClimateState,
    hydrology: &HydrologyState,
    geology: &GeologyState,
) -> (SoilState, VegetationState) {
    let mut soil_kind = Vec::with_capacity(grid.len());
    let mut depth_m = Vec::with_capacity(grid.len());
    let mut fertility = Vec::with_capacity(grid.len());
    let mut clay_fraction = Vec::with_capacity(grid.len());
    let mut organic_fraction = Vec::with_capacity(grid.len());
    let mut drainage = Vec::with_capacity(grid.len());
    let mut biome = Vec::with_capacity(grid.len());
    let mut canopy_fraction = Vec::with_capacity(grid.len());
    let mut grass_fraction = Vec::with_capacity(grid.len());
    let mut biomass = Vec::with_capacity(grid.len());
    let mut fire_frequency = Vec::with_capacity(grid.len());

    for index in 0..grid.len() {
        let elevation = tectonics.elevation_m[index];
        let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
        let temperature = climate.temperature_c[index];
        let precipitation = climate.precipitation_mm[index];
        let aridity = climate.aridity[index];
        let wetness = hydrology.wetness[index];
        let runoff = hydrology.runoff[index];
        let river = hydrology.river_strength[index];
        let lake = hydrology.lake[index];
        let erosion = hydrology.erosion_m[index];
        let sediment = geology.sediment_m[index];
        let ash = geology.volcanic_ash_m[index];
        let volcanism = tectonics.volcanism[index];
        let weathering = geology.weathering[index];
        let is_ocean = elevation <= terrain.sea_level_m;

        let floodplain = (river * (1.0 - smoothstep(0.004, 0.045, slope))
            + smoothstep(2.0, 35.0, sediment) * 0.34)
            .clamp(0.0, 1.0);
        let waterlogging =
            (wetness * (1.0 - smoothstep(0.004, 0.08, slope)) + lake * 0.72).clamp(0.0, 1.0);
        let drainage_value = (smoothstep(0.001, 0.10, slope) * 0.50
            + (1.0 - waterlogging) * 0.38
            + (1.0 - smoothstep(0.18, 0.68, weathering)) * 0.12)
            .clamp(0.0, 1.0);
        let clay = (weathering * 0.48
            + smoothstep(1.5, 48.0, sediment) * 0.30
            + smoothstep(0.08, 2.5, ash) * 0.15
            + floodplain * 0.08)
            .clamp(0.02, 0.82);
        let slope_retention = 1.0 - smoothstep(0.055, 0.28, slope);
        let erosion_loss = smoothstep(12.0, 260.0, erosion) * 0.48;
        let base_depth =
            (weathering * 2.5 + sediment.ln_1p() * 0.34 + ash.min(8.0) * 0.11 + floodplain * 0.82)
                * slope_retention
                * (1.0 - erosion_loss);
        let moisture_available =
            ((1.0 - aridity) * 0.68 + wetness * 0.22 + runoff * 0.10).clamp(0.0, 1.0);
        let thermal_growth = smoothstep(-7.0, 16.0, temperature)
            * (1.0 - smoothstep(32.0, 43.0, temperature) * 0.28);
        let provisional_biomass = if is_ocean {
            0.0
        } else {
            thermal_growth * moisture_available.powf(1.16)
        };
        let cool_retention = 1.0 - smoothstep(14.0, 31.0, temperature) * 0.42;
        let organic = (provisional_biomass
            * (0.20 + waterlogging * 0.46 + floodplain * 0.12)
            * cool_retention
            + f32::from(temperature < 7.0) * waterlogging * 0.20)
            .clamp(0.0, 0.72);
        let moderate_weathering = weathering * (1.0 - weathering * 0.52);
        let fertility_value = (moderate_weathering * 0.34
            + organic * 0.76
            + smoothstep(0.04, 1.2, ash) * 0.20
            + floodplain * 0.24
            - aridity * 0.13
            - waterlogging.powi(3) * 0.06)
            .clamp(0.0, 1.0);
        let kind = classify_soil(
            is_ocean,
            temperature,
            precipitation,
            aridity,
            wetness,
            slope,
            ash,
            volcanism,
            sediment,
            organic,
            drainage_value,
            base_depth,
            weathering,
            river,
            lake,
        );
        let biome_value = classify_biome(
            is_ocean,
            elevation,
            temperature,
            precipitation,
            aridity,
            wetness,
            slope,
            climate.seasonality[index],
            river,
            lake,
        );
        let (canopy, grass) = vegetation_cover(
            biome_value,
            fertility_value,
            moisture_available,
            thermal_growth,
        );
        let biomass_value = (canopy * 0.82 + grass * 0.48 + organic * 0.22).clamp(0.0, 1.0);
        let fire = (((grass * 0.58 + canopy * 0.14)
            * smoothstep(0.40, 0.76, aridity)
            * smoothstep(8.0, 28.0, temperature))
            * (0.55 + climate.seasonality[index] * 0.45)
            * (1.0 - waterlogging * 0.74))
            .clamp(0.0, 1.0);

        soil_kind.push(kind as u8);
        depth_m.push(if is_ocean {
            0.0
        } else {
            base_depth.clamp(0.02, 6.0)
        });
        fertility.push(fertility_value);
        clay_fraction.push(if is_ocean { 0.0 } else { clay });
        organic_fraction.push(organic);
        drainage.push(if is_ocean { 0.0 } else { drainage_value });
        biome.push(biome_value as u8);
        canopy_fraction.push(canopy);
        grass_fraction.push(grass);
        biomass.push(biomass_value);
        fire_frequency.push(fire);
    }

    (
        SoilState {
            kind: soil_kind,
            depth_m,
            fertility,
            clay_fraction,
            organic_fraction,
            drainage,
        },
        VegetationState {
            biome,
            canopy_fraction,
            grass_fraction,
            biomass,
            fire_frequency,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn classify_soil(
    ocean: bool,
    temperature: f32,
    precipitation: f32,
    aridity: f32,
    wetness: f32,
    slope: f32,
    ash: f32,
    volcanism: f32,
    sediment: f32,
    organic: f32,
    drainage: f32,
    depth: f32,
    weathering: f32,
    river: f32,
    lake: f32,
) -> SoilKind {
    if ocean {
        SoilKind::Ocean
    } else if slope > 0.24 || (slope > 0.10 && depth < 0.16) {
        SoilKind::BareRock
    } else if temperature < -7.5 {
        SoilKind::Cryosol
    } else if wetness > 0.38 && drainage < 0.40 && organic > 0.12 {
        SoilKind::Peat
    } else if ash > 0.16 || (volcanism > 0.62 && weathering > 0.04) {
        SoilKind::Volcanic
    } else if slope < 0.060 && wetness > 0.10 && (river > 0.06 || sediment > 1.0) {
        SoilKind::Alluvial
    } else if aridity > 0.73
        && slope < 0.030
        && (lake > 0.005 || (drainage < 0.50 && sediment > 0.45))
    {
        SoilKind::Saline
    } else if aridity > 0.72 || precipitation < 220.0 {
        SoilKind::Desert
    } else if temperature > 20.0 && precipitation > 1_250.0 && weathering > 0.42 {
        SoilKind::Laterite
    } else if precipitation > 680.0 && aridity < 0.68 {
        SoilKind::Forest
    } else {
        SoilKind::Chernozem
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_biome(
    ocean: bool,
    elevation: f32,
    temperature: f32,
    precipitation: f32,
    aridity: f32,
    wetness: f32,
    slope: f32,
    seasonality: f32,
    river: f32,
    lake: f32,
) -> Biome {
    if ocean {
        Biome::Ocean
    } else if temperature < -12.0 {
        Biome::Ice
    } else if temperature < 2.0 {
        Biome::Tundra
    } else if elevation > 3_650.0 || slope > 0.28 {
        Biome::Alpine
    } else if slope < 0.045 && wetness > 0.28 && (river > 0.06 || lake > 0.025) {
        Biome::Wetland
    } else if aridity > 0.75 || precipitation < 170.0 {
        Biome::Desert
    } else if temperature < 8.0 && precipitation > 420.0 && aridity < 0.70 {
        Biome::BorealForest
    } else if temperature < 8.0 {
        Biome::TemperateGrassland
    } else if temperature > 23.0 && precipitation > 1_750.0 && aridity < 0.58 {
        Biome::TropicalRainforest
    } else if temperature > 20.0 && precipitation > 850.0 && aridity < 0.68 {
        Biome::TropicalSeasonalForest
    } else if temperature > 11.0
        && seasonality > 0.35
        && aridity > 0.52
        && (320.0..1_100.0).contains(&precipitation)
    {
        Biome::Mediterranean
    } else if temperature > 17.0 && aridity > 0.57 {
        Biome::Savanna
    } else if precipitation > 620.0 && aridity < 0.67 {
        Biome::TemperateForest
    } else {
        Biome::TemperateGrassland
    }
}

fn vegetation_cover(
    biome: Biome,
    fertility: f32,
    moisture: f32,
    thermal_growth: f32,
) -> (f32, f32) {
    let (canopy, grass) = match biome {
        Biome::Ocean | Biome::Ice | Biome::Alpine => (0.0, 0.0),
        Biome::Tundra => (0.02, 0.22),
        Biome::BorealForest => (0.62, 0.18),
        Biome::TemperateForest => (0.78, 0.28),
        Biome::TemperateGrassland => (0.12, 0.84),
        Biome::Mediterranean => (0.34, 0.56),
        Biome::Desert => (0.01, 0.08),
        Biome::Savanna => (0.24, 0.79),
        Biome::TropicalSeasonalForest => (0.69, 0.39),
        Biome::TropicalRainforest => (0.94, 0.24),
        Biome::Wetland => (0.28, 0.72),
    };
    let growth_support = (0.26 + fertility * 0.38 + moisture * 0.36) * thermal_growth.max(0.16);
    (
        (canopy * growth_support).clamp(0.0, 1.0),
        (grass * (0.30 + fertility * 0.28 + moisture * 0.42) * thermal_growth.max(0.32))
            .clamp(0.0, 1.0),
    )
}
