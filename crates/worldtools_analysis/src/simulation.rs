use serde::{Deserialize, Serialize};
use worldtools_simulation::{
    Biome, Lithology, ResourceDeposit, SoilKind, WorldSample, WorldSnapshot,
};

const RIVER_STRENGTH_THRESHOLD: f32 = 0.2;
const ACTIVE_VOLCANISM_THRESHOLD: f32 = 0.45;
const VOLCANIC_UPLIFT_THRESHOLD_M: f32 = 300.0;
const NUMERIC_FIELDS_PER_CELL: usize = 39;

/// A conditional count with equirectangular cell-area compensation.
///
/// `basis` is the population being tested and `matching` is the portion that
/// satisfies the relationship. Raw counts are retained for debugging. The
/// weighted fraction is preferred for global comparisons because atlas rows
/// represent progressively less surface area toward the poles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CoherenceMeasure {
    pub basis_cells: usize,
    pub matching_cells: usize,
    pub basis_area_weight: f64,
    pub matching_area_weight: f64,
}

impl CoherenceMeasure {
    /// Returns the area-weighted matching fraction, or `None` when the basis
    /// population is empty.
    #[must_use]
    pub fn area_weighted_fraction(self) -> Option<f64> {
        (self.basis_area_weight > 0.0).then(|| self.matching_area_weight / self.basis_area_weight)
    }

    #[must_use]
    pub const fn has_observations(self) -> bool {
        self.basis_cells != 0
    }

    fn observe(&mut self, in_basis: bool, matches: bool, area_weight: f64) {
        if !in_basis {
            return;
        }
        self.basis_cells += 1;
        self.basis_area_weight += area_weight;
        if matches {
            self.matching_cells += 1;
            self.matching_area_weight += area_weight;
        }
    }
}

/// Host-process evidence for one deposit family whose occurrence is directly
/// constrained by fields in the world-history model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceProcessAudit {
    pub deposit: ResourceDeposit,
    pub hosted: CoherenceMeasure,
}

/// Cross-layer coherence report for an immutable world-history snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldSimulationAudit {
    pub atlas_cells: usize,
    pub numeric_values_checked: usize,
    pub non_finite_numeric_values: usize,
    pub cells_with_non_finite_values: usize,
    /// Ocean cells whose soil is classified as marine sediment.
    pub ocean_soil: CoherenceMeasure,
    /// Ocean cells whose vegetation layer is classified as ocean.
    pub ocean_biome: CoherenceMeasure,
    /// Land cells containing a resolved river channel.
    pub land_rivers: CoherenceMeasure,
    /// Volcanically active cells with substantial positive uplift.
    pub volcanic_relief: CoherenceMeasure,
    /// Process checks for deposits represented by explicit host controls.
    pub resource_processes: Vec<ResourceProcessAudit>,
}

impl WorldSimulationAudit {
    #[must_use]
    pub const fn is_finite(&self) -> bool {
        self.non_finite_numeric_values == 0 && self.cells_with_non_finite_values == 0
    }

    #[must_use]
    pub fn resource_process(&self, deposit: ResourceDeposit) -> Option<&ResourceProcessAudit> {
        self.resource_processes
            .iter()
            .find(|audit| audit.deposit == deposit)
    }
}

/// Evaluates numerical integrity and process coherence across one complete
/// global simulation atlas.
///
/// Samples are read at atlas cell centers. Counts are therefore exact for the
/// categorical fields stored by [`WorldSnapshot`], while weighted ratios
/// compensate for the equirectangular atlas's latitude-dependent cell area.
#[must_use]
pub fn audit_world_snapshot(snapshot: &WorldSnapshot) -> WorldSimulationAudit {
    let grid = snapshot.grid();
    let mut report = WorldSimulationAudit {
        atlas_cells: grid.len(),
        numeric_values_checked: 0,
        non_finite_numeric_values: 0,
        cells_with_non_finite_values: 0,
        ocean_soil: CoherenceMeasure::default(),
        ocean_biome: CoherenceMeasure::default(),
        land_rivers: CoherenceMeasure::default(),
        volcanic_relief: CoherenceMeasure::default(),
        resource_processes: audited_resource_deposits()
            .map(|deposit| ResourceProcessAudit {
                deposit,
                hosted: CoherenceMeasure::default(),
            })
            .collect(),
    };
    let sea_level_m = snapshot.terrain_settings().sea_level_m;

    for index in 0..grid.len() {
        let point = grid.point(index);
        let sample = snapshot.sample(point);
        let area_weight = point.latitude.cos().max(0.0);
        let values = numeric_values(sample);
        let non_finite = values.iter().filter(|value| !value.is_finite()).count();
        report.numeric_values_checked += values.len();
        report.non_finite_numeric_values += non_finite;
        report.cells_with_non_finite_values += usize::from(non_finite != 0);

        let ocean = sample.elevation_m <= sea_level_m;
        report
            .ocean_soil
            .observe(ocean, sample.soil.kind == SoilKind::Ocean, area_weight);
        report
            .ocean_biome
            .observe(ocean, sample.vegetation.biome == Biome::Ocean, area_weight);
        report.land_rivers.observe(
            !ocean,
            sample.hydrology.river_strength >= RIVER_STRENGTH_THRESHOLD,
            area_weight,
        );
        report.volcanic_relief.observe(
            sample.tectonics.volcanism >= ACTIVE_VOLCANISM_THRESHOLD,
            sample.tectonics.uplift_m >= VOLCANIC_UPLIFT_THRESHOLD_M,
            area_weight,
        );

        for process in &mut report.resource_processes {
            process.hosted.observe(
                sample.resources.dominant == process.deposit,
                resource_has_process_host(process.deposit, sample),
                area_weight,
            );
        }
    }

    debug_assert_eq!(
        report.numeric_values_checked,
        report.atlas_cells * NUMERIC_FIELDS_PER_CELL
    );
    report
}

fn audited_resource_deposits() -> impl Iterator<Item = ResourceDeposit> {
    [
        ResourceDeposit::PorphyryCopper,
        ResourceDeposit::Bauxite,
        ResourceDeposit::Peat,
        ResourceDeposit::RockSalt,
        ResourceDeposit::Nitrate,
    ]
    .into_iter()
}

fn resource_has_process_host(deposit: ResourceDeposit, sample: WorldSample) -> bool {
    match deposit {
        ResourceDeposit::PorphyryCopper => {
            sample.tectonics.convergence >= 0.15
                && sample.tectonics.volcanism >= 0.15
                && matches!(
                    sample.geology.lithology,
                    Lithology::VolcanicArc | Lithology::Plutonic
                )
        }
        ResourceDeposit::Bauxite => {
            sample.climate.temperature_c >= 15.0
                && sample.climate.precipitation_mm >= 700.0
                && sample.geology.weathering >= 0.1
                && matches!(
                    sample.geology.lithology,
                    Lithology::FelsicCraton | Lithology::VolcanicArc
                )
        }
        ResourceDeposit::Peat => {
            sample.soil.kind == SoilKind::Peat
                && sample.soil.organic_fraction >= 0.2
                && sample.hydrology.wetness >= 0.2
        }
        ResourceDeposit::RockSalt => {
            sample.climate.aridity >= 0.6 && sample.geology.sediment_m >= 10.0
        }
        ResourceDeposit::Nitrate => {
            sample.climate.aridity >= 0.6
                && matches!(sample.soil.kind, SoilKind::Desert | SoilKind::Saline)
                && sample.hydrology.runoff <= 0.8
        }
        _ => false,
    }
}

fn numeric_values(sample: WorldSample) -> [f32; NUMERIC_FIELDS_PER_CELL] {
    [
        sample.elevation_m,
        sample.slope,
        sample.tectonics.crust_age_myr,
        sample.tectonics.boundary,
        sample.tectonics.convergence,
        sample.tectonics.divergence,
        sample.tectonics.volcanism,
        sample.tectonics.uplift_m,
        sample.hydrology.runoff,
        sample.hydrology.river_strength,
        sample.hydrology.wetness,
        sample.hydrology.lake,
        sample.hydrology.erosion_m,
        sample.hydrology.sediment_m,
        sample.climate.temperature_c,
        sample.climate.precipitation_mm,
        sample.climate.seasonality,
        sample.climate.wind_east,
        sample.climate.wind_north,
        sample.climate.aridity,
        sample.soil.depth_m,
        sample.soil.fertility,
        sample.soil.clay_fraction,
        sample.soil.organic_fraction,
        sample.soil.drainage,
        sample.vegetation.canopy_fraction,
        sample.vegetation.grass_fraction,
        sample.vegetation.biomass,
        sample.vegetation.fire_frequency,
        sample.geology.rock_age_myr,
        sample.geology.sediment_m,
        sample.geology.volcanic_ash_m,
        sample.geology.weathering,
        sample.resources.richness,
        sample.resources.depth_m,
        sample.resources.confidence,
        sample.resources.metallic,
        sample.resources.energy,
        sample.resources.industrial,
    ]
}

#[cfg(test)]
mod tests {
    use worldtools_simulation::{SimulationSettings, WorldSnapshot};
    use worldtools_world::{TerrainSettings, WorldSeed};

    use super::*;

    fn test_snapshot() -> WorldSnapshot {
        WorldSnapshot::generate(
            WorldSeed(712),
            TerrainSettings::default(),
            SimulationSettings {
                atlas_width: 72,
                atlas_height: 36,
                plate_count: 10,
                hotspot_count: 5,
                geological_age_myr: 180,
                erosion_iterations: 3,
                moisture_iterations: 8,
            },
        )
    }

    #[test]
    fn audit_reports_finite_and_coupled_world_history() {
        let audit = audit_world_snapshot(&test_snapshot());

        assert_eq!(audit.atlas_cells, 72 * 36);
        assert_eq!(
            audit.numeric_values_checked,
            audit.atlas_cells * NUMERIC_FIELDS_PER_CELL
        );
        assert!(audit.is_finite());
        assert!(audit.ocean_soil.has_observations());
        assert!(audit.ocean_biome.has_observations());
        assert!(audit.ocean_soil.area_weighted_fraction().unwrap() > 0.9);
        assert!(audit.ocean_biome.area_weighted_fraction().unwrap() > 0.9);
        assert!(audit.land_rivers.matching_cells > 0);
        assert!(audit.volcanic_relief.matching_cells > 0);

        let observed_processes = audit
            .resource_processes
            .iter()
            .filter(|process| process.hosted.has_observations())
            .collect::<Vec<_>>();
        assert!(!observed_processes.is_empty());
        assert!(
            observed_processes
                .iter()
                .all(|process| { process.hosted.area_weighted_fraction().unwrap() >= 0.95 })
        );
    }

    #[test]
    fn resource_process_contract_lists_each_audited_deposit_once() {
        let deposits = audited_resource_deposits().collect::<Vec<_>>();
        assert_eq!(deposits.len(), 5);
        for deposit in deposits {
            assert_eq!(
                audited_resource_deposits()
                    .filter(|candidate| *candidate == deposit)
                    .count(),
                1
            );
        }
    }
}
