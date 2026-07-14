use std::{fmt, sync::Arc};

use rayon::prelude::*;
use worldtools_world::{GeoPoint, TerrainGenerator, TerrainSettings, WorldSeed};

use crate::{
    AtlasGrid, SimulationSettings,
    layers::{
        Biome, BoundaryKind, ClimateSample, CrustKind, GeologySample, HydrologySample, KoppenZone,
        Lithology, ResourceDeposit, ResourcesSample, SoilKind, SoilSample, TectonicsSample,
        VegetationSample, WorldDataLayer, WorldSample,
    },
    stages::{
        evolve_surface_geology, refresh_hydrology, simulate_climate, simulate_ecology,
        simulate_glaciation, simulate_hydrology, simulate_resources, simulate_tectonics,
    },
};

const SNAPSHOT_VERSION: &str = "worldtools.simulation.snapshot.v3";

#[derive(Debug, Clone)]
struct TectonicAtlas {
    plate_id: Arc<[u16]>,
    paleo_plate_id: Arc<[u16]>,
    terrane_id: Arc<[u16]>,
    crust: Arc<[u8]>,
    boundary_kind: Arc<[u8]>,
    crust_age_myr: Arc<[f32]>,
    crust_thickness_km: Arc<[f32]>,
    boundary: Arc<[f32]>,
    convergence: Arc<[f32]>,
    divergence: Arc<[f32]>,
    shear: Arc<[f32]>,
    suture: Arc<[f32]>,
    metamorphic_grade: Arc<[f32]>,
    volcanism: Arc<[f32]>,
    uplift_m: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct HydrologyAtlas {
    runoff: Arc<[f32]>,
    river_strength: Arc<[f32]>,
    wetness: Arc<[f32]>,
    lake: Arc<[f32]>,
    erosion_m: Arc<[f32]>,
    sediment_m: Arc<[f32]>,
    maximum_ice_fraction: Arc<[f32]>,
    ice_flux: Arc<[f32]>,
    glacial_erosion_m: Arc<[f32]>,
    till_m: Arc<[f32]>,
    outwash_m: Arc<[f32]>,
    isostatic_rebound_m: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct ClimateAtlas {
    zone: Arc<[u8]>,
    temperature_c: Arc<[f32]>,
    precipitation_mm: Arc<[f32]>,
    seasonality: Arc<[f32]>,
    wind_east: Arc<[f32]>,
    wind_north: Arc<[f32]>,
    aridity: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct SoilAtlas {
    kind: Arc<[u8]>,
    depth_m: Arc<[f32]>,
    fertility: Arc<[f32]>,
    clay_fraction: Arc<[f32]>,
    organic_fraction: Arc<[f32]>,
    drainage: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct VegetationAtlas {
    biome: Arc<[u8]>,
    canopy_fraction: Arc<[f32]>,
    grass_fraction: Arc<[f32]>,
    biomass: Arc<[f32]>,
    fire_frequency: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct GeologyAtlas {
    lithology: Arc<[u8]>,
    rock_age_myr: Arc<[f32]>,
    sediment_m: Arc<[f32]>,
    volcanic_ash_m: Arc<[f32]>,
    weathering: Arc<[f32]>,
}

#[derive(Debug, Clone)]
struct ResourcesAtlas {
    dominant: Arc<[u8]>,
    richness: Arc<[f32]>,
    depth_m: Arc<[f32]>,
    confidence: Arc<[f32]>,
    metallic: Arc<[f32]>,
    energy: Arc<[f32]>,
    industrial: Arc<[f32]>,
}

/// Immutable result of the coupled pre-display world-history simulation.
///
/// Cloning a snapshot is cheap because all atlas channels use shared storage.
#[derive(Clone)]
pub struct WorldSnapshot {
    seed: WorldSeed,
    terrain_settings: TerrainSettings,
    simulation_settings: SimulationSettings,
    grid: AtlasGrid,
    fingerprint: [u8; 32],
    revision: u64,
    terrain_generator: Arc<TerrainGenerator>,
    baseline_elevation_m: Arc<[f32]>,
    elevation_m: Arc<[f32]>,
    slope: Arc<[f32]>,
    tectonics: TectonicAtlas,
    hydrology: HydrologyAtlas,
    climate: ClimateAtlas,
    soil: SoilAtlas,
    vegetation: VegetationAtlas,
    geology: GeologyAtlas,
    resources: ResourcesAtlas,
}

impl fmt::Debug for WorldSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorldSnapshot")
            .field("seed", &self.seed)
            .field("terrain_settings", &self.terrain_settings)
            .field("simulation_settings", &self.simulation_settings)
            .field("grid", &self.grid)
            .field("revision", &self.revision)
            .finish_non_exhaustive()
    }
}

impl WorldSnapshot {
    #[must_use]
    #[allow(clippy::too_many_lines)] // Stage ordering remains explicit at the composition root.
    pub fn generate(
        seed: WorldSeed,
        terrain_settings: TerrainSettings,
        simulation_settings: SimulationSettings,
    ) -> Self {
        let simulation_settings = simulation_settings.sanitized();
        let grid = AtlasGrid::new(
            simulation_settings.atlas_width,
            simulation_settings.atlas_height,
        );
        let mut tectonics = simulate_tectonics(grid, seed, terrain_settings, simulation_settings);
        let provisional_climate = simulate_climate(
            grid,
            seed,
            terrain_settings,
            simulation_settings,
            &tectonics,
        );
        let mut hydrology = simulate_hydrology(
            grid,
            terrain_settings,
            simulation_settings,
            &provisional_climate,
            &mut tectonics,
        );
        let mut climate = simulate_climate(
            grid,
            seed,
            terrain_settings,
            simulation_settings,
            &tectonics,
        );
        refresh_hydrology(grid, terrain_settings, &climate, &tectonics, &mut hydrology);
        let glaciation = simulate_glaciation(
            grid,
            terrain_settings,
            simulation_settings,
            &mut tectonics,
            &climate,
            &mut hydrology,
        );
        climate = simulate_climate(
            grid,
            seed,
            terrain_settings,
            simulation_settings,
            &tectonics,
        );
        refresh_hydrology(grid, terrain_settings, &climate, &tectonics, &mut hydrology);
        let geology =
            evolve_surface_geology(grid, terrain_settings, &tectonics, &climate, &hydrology);
        let (soil, vegetation) = simulate_ecology(
            grid,
            terrain_settings,
            &tectonics,
            &climate,
            &hydrology,
            &geology,
        );
        let resources = simulate_resources(
            grid,
            seed,
            terrain_settings,
            &tectonics,
            &climate,
            &hydrology,
            &geology,
            &soil,
            &vegetation,
        );
        let slope = (0..grid.len())
            .into_par_iter()
            .map(|index| {
                grid.slope(
                    &tectonics.elevation_m,
                    index,
                    terrain_settings.planet_radius_m,
                )
            })
            .collect::<Vec<_>>();
        let fingerprint = fingerprint(seed, terrain_settings, simulation_settings);
        let revision = u64::from_le_bytes([
            fingerprint[0],
            fingerprint[1],
            fingerprint[2],
            fingerprint[3],
            fingerprint[4],
            fingerprint[5],
            fingerprint[6],
            fingerprint[7],
        ]);

        Self {
            seed,
            terrain_settings,
            simulation_settings,
            grid,
            fingerprint,
            revision,
            terrain_generator: Arc::new(TerrainGenerator::new(seed, terrain_settings)),
            baseline_elevation_m: tectonics.baseline_elevation_m.into(),
            elevation_m: tectonics.elevation_m.into(),
            slope: slope.into(),
            tectonics: TectonicAtlas {
                plate_id: tectonics.plate_id.into(),
                paleo_plate_id: tectonics.paleo_plate_id.into(),
                terrane_id: tectonics.terrane_id.into(),
                crust: tectonics.crust.into(),
                boundary_kind: tectonics.boundary_kind.into(),
                crust_age_myr: tectonics.crust_age_myr.into(),
                crust_thickness_km: tectonics.crust_thickness_km.into(),
                boundary: tectonics.boundary.into(),
                convergence: tectonics.convergence.into(),
                divergence: tectonics.divergence.into(),
                shear: tectonics.shear.into(),
                suture: tectonics.suture.into(),
                metamorphic_grade: tectonics.metamorphic_grade.into(),
                volcanism: tectonics.volcanism.into(),
                uplift_m: tectonics.uplift_m.into(),
            },
            hydrology: HydrologyAtlas {
                runoff: hydrology.runoff.into(),
                river_strength: hydrology.river_strength.into(),
                wetness: hydrology.wetness.into(),
                lake: hydrology.lake.into(),
                erosion_m: hydrology.erosion_m.into(),
                sediment_m: hydrology.sediment_m.into(),
                maximum_ice_fraction: glaciation.maximum_ice_fraction.into(),
                ice_flux: glaciation.ice_flux.into(),
                glacial_erosion_m: glaciation.erosion_m.into(),
                till_m: glaciation.till_m.into(),
                outwash_m: glaciation.outwash_m.into(),
                isostatic_rebound_m: glaciation.rebound_m.into(),
            },
            climate: ClimateAtlas {
                zone: climate.zone.into(),
                temperature_c: climate.temperature_c.into(),
                precipitation_mm: climate.precipitation_mm.into(),
                seasonality: climate.seasonality.into(),
                wind_east: climate.wind_east.into(),
                wind_north: climate.wind_north.into(),
                aridity: climate.aridity.into(),
            },
            soil: SoilAtlas {
                kind: soil.kind.into(),
                depth_m: soil.depth_m.into(),
                fertility: soil.fertility.into(),
                clay_fraction: soil.clay_fraction.into(),
                organic_fraction: soil.organic_fraction.into(),
                drainage: soil.drainage.into(),
            },
            vegetation: VegetationAtlas {
                biome: vegetation.biome.into(),
                canopy_fraction: vegetation.canopy_fraction.into(),
                grass_fraction: vegetation.grass_fraction.into(),
                biomass: vegetation.biomass.into(),
                fire_frequency: vegetation.fire_frequency.into(),
            },
            geology: GeologyAtlas {
                lithology: geology.lithology.into(),
                rock_age_myr: geology.rock_age_myr.into(),
                sediment_m: geology.sediment_m.into(),
                volcanic_ash_m: geology.volcanic_ash_m.into(),
                weathering: geology.weathering.into(),
            },
            resources: ResourcesAtlas {
                dominant: resources.dominant.into(),
                richness: resources.richness.into(),
                depth_m: resources.depth_m.into(),
                confidence: resources.confidence.into(),
                metallic: resources.metallic.into(),
                energy: resources.energy.into(),
                industrial: resources.industrial.into(),
            },
        }
    }

    #[must_use]
    pub const fn seed(&self) -> WorldSeed {
        self.seed
    }

    #[must_use]
    pub const fn terrain_settings(&self) -> TerrainSettings {
        self.terrain_settings
    }

    #[must_use]
    pub const fn simulation_settings(&self) -> SimulationSettings {
        self.simulation_settings
    }

    #[must_use]
    pub const fn grid(&self) -> AtlasGrid {
        self.grid
    }

    #[must_use]
    pub const fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn elevation_m(&self) -> &[f32] {
        &self.elevation_m
    }

    /// Samples procedural local relief plus the bilinear deformation accumulated
    /// by global tectonic, volcanic, depositional, and erosion stages.
    #[must_use]
    pub fn sample_elevation(&self, point: GeoPoint) -> f32 {
        let detailed_baseline = self.terrain_generator.sample_geo(point);
        let atlas_baseline = self.grid.sample_scalar(&self.baseline_elevation_m, point);
        let evolved_atlas = self.grid.sample_scalar(&self.elevation_m, point);
        detailed_baseline + (evolved_atlas - atlas_baseline)
    }

    /// Samples the evolved simulation surface without adding sub-atlas terrain detail.
    /// This is intended for low-resolution fallback pages during initial streaming.
    #[must_use]
    pub fn sample_atlas_elevation(&self, point: GeoPoint) -> f32 {
        self.grid.sample_scalar(&self.elevation_m, point)
    }

    #[must_use]
    pub fn sample_slope(&self, point: GeoPoint) -> f32 {
        self.grid.sample_scalar(&self.slope, point)
    }

    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    #[must_use]
    pub fn sample(&self, point: GeoPoint) -> WorldSample {
        let nearest = self.grid.nearest_index(point);
        let scalar = |values: &[f32]| self.grid.sample_scalar(values, point);
        WorldSample {
            elevation_m: self.sample_elevation(point),
            slope: scalar(&self.slope),
            tectonics: TectonicsSample {
                plate_id: self.tectonics.plate_id[nearest],
                paleo_plate_id: self.tectonics.paleo_plate_id[nearest],
                terrane_id: self.tectonics.terrane_id[nearest],
                crust: CrustKind::from_byte(self.tectonics.crust[nearest]),
                boundary_kind: BoundaryKind::from_byte(self.tectonics.boundary_kind[nearest]),
                crust_age_myr: scalar(&self.tectonics.crust_age_myr),
                crust_thickness_km: scalar(&self.tectonics.crust_thickness_km),
                boundary: scalar(&self.tectonics.boundary),
                convergence: scalar(&self.tectonics.convergence),
                divergence: scalar(&self.tectonics.divergence),
                shear: scalar(&self.tectonics.shear),
                suture: scalar(&self.tectonics.suture),
                metamorphic_grade: scalar(&self.tectonics.metamorphic_grade),
                volcanism: scalar(&self.tectonics.volcanism),
                uplift_m: scalar(&self.tectonics.uplift_m),
            },
            hydrology: HydrologySample {
                runoff: scalar(&self.hydrology.runoff),
                river_strength: scalar(&self.hydrology.river_strength),
                wetness: scalar(&self.hydrology.wetness),
                lake: scalar(&self.hydrology.lake),
                erosion_m: scalar(&self.hydrology.erosion_m),
                sediment_m: scalar(&self.hydrology.sediment_m),
                maximum_ice_fraction: scalar(&self.hydrology.maximum_ice_fraction),
                ice_flux: scalar(&self.hydrology.ice_flux),
                glacial_erosion_m: scalar(&self.hydrology.glacial_erosion_m),
                till_m: scalar(&self.hydrology.till_m),
                outwash_m: scalar(&self.hydrology.outwash_m),
                isostatic_rebound_m: scalar(&self.hydrology.isostatic_rebound_m),
            },
            climate: ClimateSample {
                zone: KoppenZone::from_byte(self.climate.zone[nearest]),
                temperature_c: scalar(&self.climate.temperature_c),
                precipitation_mm: scalar(&self.climate.precipitation_mm),
                seasonality: scalar(&self.climate.seasonality),
                wind_east: scalar(&self.climate.wind_east),
                wind_north: scalar(&self.climate.wind_north),
                aridity: scalar(&self.climate.aridity),
            },
            soil: SoilSample {
                kind: SoilKind::from_byte(self.soil.kind[nearest]),
                depth_m: scalar(&self.soil.depth_m),
                fertility: scalar(&self.soil.fertility),
                clay_fraction: scalar(&self.soil.clay_fraction),
                organic_fraction: scalar(&self.soil.organic_fraction),
                drainage: scalar(&self.soil.drainage),
            },
            vegetation: VegetationSample {
                biome: Biome::from_byte(self.vegetation.biome[nearest]),
                canopy_fraction: scalar(&self.vegetation.canopy_fraction),
                grass_fraction: scalar(&self.vegetation.grass_fraction),
                biomass: scalar(&self.vegetation.biomass),
                fire_frequency: scalar(&self.vegetation.fire_frequency),
            },
            geology: GeologySample {
                lithology: Lithology::from_byte(self.geology.lithology[nearest]),
                rock_age_myr: scalar(&self.geology.rock_age_myr),
                sediment_m: scalar(&self.geology.sediment_m),
                volcanic_ash_m: scalar(&self.geology.volcanic_ash_m),
                weathering: scalar(&self.geology.weathering),
            },
            resources: ResourcesSample {
                dominant: ResourceDeposit::from_byte(self.resources.dominant[nearest]),
                richness: scalar(&self.resources.richness),
                depth_m: scalar(&self.resources.depth_m),
                confidence: scalar(&self.resources.confidence),
                metallic: scalar(&self.resources.metallic),
                energy: scalar(&self.resources.energy),
                industrial: scalar(&self.resources.industrial),
            },
        }
    }

    /// Samples only the four channels needed for one renderer view.
    #[must_use]
    pub fn sample_layer(&self, point: GeoPoint, layer: WorldDataLayer) -> [f32; 4] {
        let nearest = self.grid.nearest_index(point);
        let scalar = |values: &[f32]| self.grid.sample_scalar(values, point);
        match layer {
            WorldDataLayer::Elevation => {
                [self.sample_elevation(point), scalar(&self.slope), 0.0, 1.0]
            }
            WorldDataLayer::Tectonics => [
                f32::from(self.tectonics.plate_id[nearest]),
                scalar(&self.tectonics.convergence) - scalar(&self.tectonics.divergence),
                scalar(&self.tectonics.uplift_m),
                scalar(&self.tectonics.volcanism),
            ],
            WorldDataLayer::Hydrology => [
                scalar(&self.hydrology.river_strength),
                scalar(&self.hydrology.wetness).max(scalar(&self.hydrology.lake)),
                scalar(&self.hydrology.sediment_m),
                if scalar(&self.hydrology.maximum_ice_fraction) > 0.02 {
                    -scalar(&self.hydrology.maximum_ice_fraction)
                } else {
                    scalar(&self.hydrology.erosion_m)
                },
            ],
            WorldDataLayer::Climate => [
                scalar(&self.climate.temperature_c),
                scalar(&self.climate.precipitation_mm),
                scalar(&self.climate.wind_east),
                scalar(&self.climate.wind_north),
            ],
            WorldDataLayer::Soil => [
                f32::from(self.soil.kind[nearest]),
                scalar(&self.soil.depth_m),
                scalar(&self.soil.fertility),
                scalar(&self.soil.organic_fraction),
            ],
            WorldDataLayer::Vegetation => [
                f32::from(self.vegetation.biome[nearest]),
                scalar(&self.vegetation.canopy_fraction),
                scalar(&self.vegetation.grass_fraction),
                scalar(&self.vegetation.biomass),
            ],
            WorldDataLayer::Geology => [
                f32::from(self.geology.lithology[nearest]),
                scalar(&self.geology.rock_age_myr),
                scalar(&self.geology.sediment_m),
                scalar(&self.geology.volcanic_ash_m),
            ],
            WorldDataLayer::Resources => [
                f32::from(self.resources.dominant[nearest]),
                scalar(&self.resources.richness),
                (scalar(&self.resources.depth_m) / 6_000.0).clamp(0.0, 1.0),
                scalar(&self.resources.confidence),
            ],
        }
    }
}

fn fingerprint(
    seed: WorldSeed,
    terrain: TerrainSettings,
    simulation: SimulationSettings,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key(SNAPSHOT_VERSION);
    hasher.update(&seed.0.to_le_bytes());
    hasher.update(&terrain.fingerprint());
    simulation.hash_into(&mut hasher);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> SimulationSettings {
        SimulationSettings {
            atlas_width: 72,
            atlas_height: 36,
            climate_width: 36,
            climate_height: 18,
            plate_count: 10,
            hotspot_count: 5,
            geological_age_myr: 180,
            erosion_iterations: 3,
            moisture_iterations: 8,
            glacial_iterations: 2,
        }
    }

    #[test]
    fn generation_and_sampling_are_bit_deterministic() {
        let first = WorldSnapshot::generate(
            WorldSeed(9_1842),
            TerrainSettings::default(),
            test_settings(),
        );
        let second = WorldSnapshot::generate(
            WorldSeed(9_1842),
            TerrainSettings::default(),
            test_settings(),
        );
        assert_eq!(first.fingerprint(), second.fingerprint());
        for point in [
            GeoPoint::from_degrees(0.0, 0.0),
            GeoPoint::from_degrees(46.0, -122.0),
            GeoPoint::from_degrees(-28.0, 151.0),
        ] {
            for layer in WorldDataLayer::ALL {
                let left = first.sample_layer(point, layer).map(f32::to_bits);
                let right = second.sample_layer(point, layer).map(f32::to_bits);
                assert_eq!(left, right, "layer {layer:?} was not deterministic");
            }
        }
    }

    #[test]
    fn typed_and_direct_layer_channels_share_one_contract() {
        let snapshot =
            WorldSnapshot::generate(WorldSeed(51), TerrainSettings::default(), test_settings());
        let point = GeoPoint::from_degrees(31.25, -74.5);
        let sample = snapshot.sample(point);
        for layer in WorldDataLayer::ALL {
            assert_eq!(
                sample.layer_channels(layer).map(f32::to_bits),
                snapshot.sample_layer(point, layer).map(f32::to_bits),
                "channel contract differed for {layer:?}",
            );
        }
        assert_eq!(
            sample.layer_channels(WorldDataLayer::Tectonics)[0].to_bits(),
            f32::from(sample.tectonics.plate_id).to_bits(),
        );
    }

    #[test]
    fn every_layer_is_finite_and_spatially_varied() {
        let snapshot =
            WorldSnapshot::generate(WorldSeed(44), TerrainSettings::default(), test_settings());
        for layer in WorldDataLayer::ALL {
            let samples = (0..snapshot.grid().len())
                .step_by(7)
                .map(|index| snapshot.sample_layer(snapshot.grid().point(index), layer))
                .collect::<Vec<_>>();
            assert!(samples.iter().flatten().all(|value| value.is_finite()));
            let varied = (0..4).any(|channel| {
                let first = samples[0][channel];
                samples
                    .iter()
                    .any(|sample| (sample[channel] - first).abs() > 1.0e-5)
            });
            assert!(varied, "layer {layer:?} was spatially constant");
        }
    }

    #[test]
    fn coupled_state_respects_basic_physical_invariants() {
        let terrain = TerrainSettings::default();
        let snapshot = WorldSnapshot::generate(WorldSeed(712), terrain, test_settings());
        let mut ocean_cells = 0;
        let mut ocean_soils = 0;
        let mut active_rivers = 0;
        let mut volcanic_relief = 0;
        let mut prevailing_wind_cells = 0;
        for index in 0..snapshot.grid().len() {
            let sample = snapshot.sample(snapshot.grid().point(index));
            if sample.elevation_m <= terrain.sea_level_m {
                ocean_cells += 1;
                ocean_soils += usize::from(sample.soil.kind == SoilKind::Ocean);
            }
            active_rivers += usize::from(sample.hydrology.river_strength > 0.25);
            volcanic_relief +=
                usize::from(sample.tectonics.volcanism > 0.5 && sample.tectonics.uplift_m > 300.0);
            let wind_speed = sample.climate.wind_east.hypot(sample.climate.wind_north);
            prevailing_wind_cells += usize::from(wind_speed >= 3.0);
            assert!(wind_speed <= 16.0);
            assert!((0.0..=1.0).contains(&sample.soil.fertility));
            assert!((0.0..=1.0).contains(&sample.vegetation.biomass));
            assert!(sample.climate.precipitation_mm >= 0.0);
        }
        assert!(ocean_cells > 0);
        assert!(ocean_soils * 10 > ocean_cells * 9);
        assert!(active_rivers > 0);
        assert!(volcanic_relief > 0);
        assert!(prevailing_wind_cells * 2 > snapshot.grid().len());
    }

    #[test]
    fn scalar_sampling_is_continuous_across_antimeridian() {
        let snapshot =
            WorldSnapshot::generate(WorldSeed(88), TerrainSettings::default(), test_settings());
        let west = snapshot.sample(GeoPoint::from_degrees(12.0, -179.999));
        let east = snapshot.sample(GeoPoint::from_degrees(12.0, 179.999));
        assert!((west.elevation_m - east.elevation_m).abs() < 5.0);
        assert!((west.climate.temperature_c - east.climate.temperature_c).abs() < 0.1);
    }

    #[test]
    fn detailed_relief_survives_below_atlas_cell_scale() {
        let snapshot =
            WorldSnapshot::generate(WorldSeed(108), TerrainSettings::default(), test_settings());
        let first = GeoPoint::from_degrees(22.0, 41.0);
        let nearby = GeoPoint::from_degrees(22.015, 41.02);
        assert_eq!(
            snapshot.grid().nearest_index(first),
            snapshot.grid().nearest_index(nearby),
        );
        let first_height = snapshot.sample_elevation(first);
        let nearby_height = snapshot.sample_elevation(nearby);
        assert_ne!(first_height.to_bits(), nearby_height.to_bits());
        assert_eq!(
            first_height.to_bits(),
            snapshot.sample_elevation(first).to_bits(),
        );
    }
}
