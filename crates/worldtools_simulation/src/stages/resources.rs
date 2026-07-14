use worldtools_world::{TerrainSettings, WorldSeed};

use crate::{
    AtlasGrid,
    layers::{BoundaryKind, Lithology, ResourceDeposit, SoilKind},
    random::hash_unit,
    stages::{
        climate::ClimateState,
        ecology::{SoilState, VegetationState},
        geology::GeologyState,
        hydrology::HydrologyState,
        math::{Vec3, direction, smoothstep},
        tectonics::TectonicState,
    },
};

const REFERENCE_ATLAS_HEIGHT: f32 = 192.0;
const DEPOSIT_COUNT: usize = 15;
const METALLIC_END: usize = 7;
const ENERGY_END: usize = 11;

const DISTRICT_GATE_DOMAIN: u64 = 0x4741_5445;
const DISTRICT_SITE_DOMAIN: u64 = 0x5349_5445;
const DISTRICT_AXIS_DOMAIN: u64 = 0x4158_4953;
const DISTRICT_SIZE_DOMAIN: u64 = 0x5349_5a45;
const DISTRICT_STRENGTH_DOMAIN: u64 = 0x5354_524e;
const REGIONAL_FABRIC_DOMAIN: u64 = 0x5245_474e;
const LOCAL_FABRIC_DOMAIN: u64 = 0x4c4f_434c;

#[derive(Debug)]
pub(crate) struct ResourcesState {
    pub(crate) dominant: Vec<u8>,
    pub(crate) richness: Vec<f32>,
    pub(crate) depth_m: Vec<f32>,
    pub(crate) confidence: Vec<f32>,
    pub(crate) metallic: Vec<f32>,
    pub(crate) energy: Vec<f32>,
    pub(crate) industrial: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct DepositModel {
    deposit: ResourceDeposit,
    /// Host score below which this deposit cannot occur.
    host_floor: f32,
    /// Localized process score required for a mapped occurrence.
    occurrence_floor: f32,
    /// Localized score represented as a fully rich occurrence.
    rich_score: f32,
    /// Reference-grid distance between candidate mineral districts.
    spacing_cells: f32,
    /// Long and short footprint radii on the 192-row reference atlas.
    major_radius_cells: f32,
    minor_radius_cells: f32,
    /// Fraction of eligible regional bins that can contain a district.
    frequency: f32,
    /// Deposits from the same family inherit the same broad structural fabric.
    family: u8,
}

const DEPOSIT_MODELS: [DepositModel; DEPOSIT_COUNT] = [
    DepositModel {
        deposit: ResourceDeposit::BandedIron,
        host_floor: 0.20,
        occurrence_floor: 0.16,
        rich_score: 0.72,
        spacing_cells: 24.0,
        major_radius_cells: 5.2,
        minor_radius_cells: 2.0,
        frequency: 0.48,
        family: 1,
    },
    DepositModel {
        deposit: ResourceDeposit::Bauxite,
        host_floor: 0.20,
        occurrence_floor: 0.13,
        rich_score: 0.52,
        spacing_cells: 11.0,
        major_radius_cells: 3.2,
        minor_radius_cells: 1.6,
        frequency: 0.65,
        family: 2,
    },
    DepositModel {
        deposit: ResourceDeposit::PorphyryCopper,
        host_floor: 0.20,
        occurrence_floor: 0.12,
        rich_score: 0.50,
        spacing_cells: 10.0,
        major_radius_cells: 2.5,
        minor_radius_cells: 0.9,
        frequency: 0.56,
        family: 3,
    },
    DepositModel {
        deposit: ResourceDeposit::VolcanogenicSulfide,
        host_floor: 0.11,
        occurrence_floor: 0.07,
        rich_score: 0.34,
        spacing_cells: 9.0,
        major_radius_cells: 2.6,
        minor_radius_cells: 0.9,
        frequency: 0.58,
        family: 4,
    },
    DepositModel {
        deposit: ResourceDeposit::Nickel,
        host_floor: 0.25,
        occurrence_floor: 0.18,
        rich_score: 0.62,
        spacing_cells: 16.0,
        major_radius_cells: 1.9,
        minor_radius_cells: 0.7,
        frequency: 0.28,
        family: 5,
    },
    DepositModel {
        deposit: ResourceDeposit::Gold,
        host_floor: 0.20,
        occurrence_floor: 0.15,
        rich_score: 0.58,
        spacing_cells: 14.0,
        major_radius_cells: 2.2,
        minor_radius_cells: 0.7,
        frequency: 0.36,
        family: 6,
    },
    DepositModel {
        deposit: ResourceDeposit::Gemstones,
        host_floor: 0.14,
        occurrence_floor: 0.09,
        rich_score: 0.42,
        spacing_cells: 14.0,
        major_radius_cells: 2.2,
        minor_radius_cells: 0.9,
        frequency: 0.38,
        family: 7,
    },
    DepositModel {
        deposit: ResourceDeposit::Coal,
        host_floor: 0.09,
        occurrence_floor: 0.06,
        rich_score: 0.28,
        spacing_cells: 14.0,
        major_radius_cells: 4.2,
        minor_radius_cells: 2.0,
        frequency: 0.62,
        family: 8,
    },
    DepositModel {
        deposit: ResourceDeposit::Peat,
        host_floor: 0.40,
        occurrence_floor: 0.27,
        rich_score: 0.72,
        spacing_cells: 9.0,
        major_radius_cells: 2.4,
        minor_radius_cells: 1.2,
        frequency: 0.78,
        family: 8,
    },
    DepositModel {
        deposit: ResourceDeposit::Petroleum,
        host_floor: 0.07,
        occurrence_floor: 0.045,
        rich_score: 0.26,
        spacing_cells: 20.0,
        major_radius_cells: 6.2,
        minor_radius_cells: 2.8,
        frequency: 0.66,
        family: 9,
    },
    DepositModel {
        deposit: ResourceDeposit::NaturalGas,
        host_floor: 0.07,
        occurrence_floor: 0.045,
        rich_score: 0.28,
        spacing_cells: 20.0,
        major_radius_cells: 6.6,
        minor_radius_cells: 2.8,
        frequency: 0.66,
        family: 9,
    },
    DepositModel {
        deposit: ResourceDeposit::RockSalt,
        host_floor: 0.10,
        occurrence_floor: 0.07,
        rich_score: 0.48,
        spacing_cells: 15.0,
        major_radius_cells: 4.8,
        minor_radius_cells: 2.2,
        frequency: 0.48,
        family: 10,
    },
    DepositModel {
        deposit: ResourceDeposit::Clay,
        host_floor: 0.15,
        occurrence_floor: 0.10,
        rich_score: 0.42,
        spacing_cells: 10.0,
        major_radius_cells: 2.8,
        minor_radius_cells: 1.4,
        frequency: 0.68,
        family: 11,
    },
    DepositModel {
        deposit: ResourceDeposit::Phosphate,
        host_floor: 0.10,
        occurrence_floor: 0.065,
        rich_score: 0.28,
        spacing_cells: 14.0,
        major_radius_cells: 3.8,
        minor_radius_cells: 1.4,
        frequency: 0.58,
        family: 12,
    },
    DepositModel {
        deposit: ResourceDeposit::Nitrate,
        host_floor: 0.30,
        occurrence_floor: 0.21,
        rich_score: 0.66,
        spacing_cells: 19.0,
        major_radius_cells: 2.7,
        minor_radius_cells: 1.2,
        frequency: 0.26,
        family: 13,
    },
];

#[derive(Clone, Copy, Debug)]
struct DistrictCandidate {
    index: usize,
    center: Vec3,
    priority: f32,
    host_score: f32,
}

#[derive(Clone, Copy, Debug)]
struct District {
    center: Vec3,
    major_axis: Vec3,
    minor_axis: Vec3,
    major_sine: f64,
    minor_sine: f64,
    minimum_alignment: f64,
    strength: f32,
}

#[derive(Clone, Copy, Debug)]
struct SelectedDeposit {
    model: DepositModel,
    host_score: f32,
    richness: f32,
    district_focus: f32,
    fabric: f32,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn simulate(
    grid: AtlasGrid,
    seed: WorldSeed,
    terrain: TerrainSettings,
    tectonics: &TectonicState,
    climate: &ClimateState,
    hydrology: &HydrologyState,
    geology: &GeologyState,
    soil: &SoilState,
    vegetation: &VegetationState,
) -> ResourcesState {
    let resource_seed = seed.key("simulation.resources.v2").u64();
    let host_scores = (0..grid.len())
        .map(|index| {
            process_scores(
                grid, index, terrain, tectonics, climate, hydrology, geology, soil, vegetation,
            )
        })
        .collect::<Vec<_>>();
    let districts = DEPOSIT_MODELS
        .iter()
        .enumerate()
        .map(|(deposit_index, model)| {
            seed_districts(grid, resource_seed, deposit_index, *model, &host_scores)
        })
        .collect::<Vec<_>>();

    let mut state = ResourcesState::with_capacity(grid.len());
    for (index, cell_host_scores) in host_scores.iter().enumerate() {
        let position = direction(grid.point(index).latitude, grid.point(index).longitude);
        let mut selected: Option<SelectedDeposit> = None;
        let mut group_prospectivity = [0.0_f32; 3];

        for (deposit_index, model) in DEPOSIT_MODELS.iter().enumerate() {
            let host_score = cell_host_scores[deposit_index];
            if host_score < model.host_floor {
                continue;
            }

            let fabric = structural_fabric(grid, index, resource_seed, *model);
            let district_focus = district_focus(position, fabric, &districts[deposit_index]);
            let localized_score = host_score * district_focus;
            let prospectivity = smoothstep(
                model.occurrence_floor * 0.45,
                model.rich_score,
                localized_score,
            );
            let group_index = usize::from(deposit_index >= METALLIC_END)
                + usize::from(deposit_index >= ENERGY_END);
            group_prospectivity[group_index] = group_prospectivity[group_index].max(prospectivity);

            if localized_score < model.occurrence_floor {
                continue;
            }

            let mapped_richness = smoothstep(
                model.occurrence_floor * 0.78,
                model.rich_score,
                localized_score,
            );
            if selected.is_none_or(|current| mapped_richness > current.richness) {
                selected = Some(SelectedDeposit {
                    model: *model,
                    host_score,
                    richness: mapped_richness,
                    district_focus,
                    fabric,
                });
            }
        }

        let (deposit, resource_richness, deposit_depth, evidence_confidence) =
            selected.map_or((ResourceDeposit::None, 0.0, 0.0, 0.0), |selected| {
                let depth = deposit_depth_m(
                    selected.model.deposit,
                    geology.sediment_m[index],
                    tectonics.uplift_m[index],
                    selected.fabric,
                );
                let host_evidence = smoothstep(
                    selected.model.host_floor,
                    (selected.model.host_floor + 0.46).min(0.92),
                    selected.host_score,
                );
                let confidence = (0.24
                    + host_evidence * 0.43
                    + selected.richness * 0.24
                    + selected.district_focus * 0.09)
                    .clamp(0.0, 1.0);
                (selected.model.deposit, selected.richness, depth, confidence)
            });

        state.dominant.push(deposit as u8);
        state.richness.push(resource_richness);
        state.depth_m.push(deposit_depth);
        state.confidence.push(evidence_confidence);
        state.metallic.push(group_prospectivity[0]);
        state.energy.push(group_prospectivity[1]);
        state.industrial.push(group_prospectivity[2]);
    }

    state
}

impl ResourcesState {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            dominant: Vec::with_capacity(capacity),
            richness: Vec::with_capacity(capacity),
            depth_m: Vec::with_capacity(capacity),
            confidence: Vec::with_capacity(capacity),
            metallic: Vec::with_capacity(capacity),
            energy: Vec::with_capacity(capacity),
            industrial: Vec::with_capacity(capacity),
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn process_scores(
    grid: AtlasGrid,
    index: usize,
    terrain: TerrainSettings,
    tectonics: &TectonicState,
    climate: &ClimateState,
    hydrology: &HydrologyState,
    geology: &GeologyState,
    soil: &SoilState,
    vegetation: &VegetationState,
) -> [f32; DEPOSIT_COUNT] {
    let lithology = Lithology::from_byte(geology.lithology[index]);
    let soil_kind = SoilKind::from_byte(soil.kind[index]);
    let age = geology.rock_age_myr[index];
    let warm = smoothstep(15.0, 29.0, climate.temperature_c[index]);
    let humid = smoothstep(700.0, 2_600.0, climate.precipitation_mm[index]);
    let arid = smoothstep(0.62, 0.9, climate.aridity[index]);
    let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
    let stable = 1.0 - tectonics.boundary[index];
    let boundary_kind = BoundaryKind::from_byte(tectonics.boundary_kind[index]);
    let subduction_arc = f32::from(matches!(
        boundary_kind,
        BoundaryKind::SubductionArc | BoundaryKind::IslandArc
    ));
    let extensional_margin = f32::from(matches!(
        boundary_kind,
        BoundaryKind::OceanRidge | BoundaryKind::ContinentalRift
    ));
    let inherited_orogen = tectonics.suture[index] * tectonics.metamorphic_grade[index];
    let ocean = tectonics.elevation_m[index] <= terrain.sea_level_m;
    let shallow_marine = smoothstep(
        terrain.sea_level_m - 650.0,
        terrain.sea_level_m - 15.0,
        tectonics.elevation_m[index],
    ) * f32::from(ocean);
    let arc_or_craton = f32::from(matches!(
        lithology,
        Lithology::FelsicCraton | Lithology::VolcanicArc
    ));
    let arc_or_pluton = f32::from(matches!(
        lithology,
        Lithology::VolcanicArc | Lithology::Plutonic
    ));
    let mafic_host = f32::from(matches!(
        lithology,
        Lithology::OceanicBasalt | Lithology::VolcanicArc
    ));
    let metamorphic_belt = f32::from(matches!(
        lithology,
        Lithology::Metamorphic | Lithology::VolcanicArc | Lithology::Plutonic
    ));
    let basin_host = if matches!(
        lithology,
        Lithology::Sedimentary | Lithology::Carbonate | Lithology::Unconsolidated
    ) {
        1.0
    } else {
        // The surface-geology pass records active cover rather than the full
        // buried column, so basement outcrop does not rule out a basin below.
        0.72
    };
    let low_relief = 1.0 - smoothstep(0.015, 0.12, slope);
    let active_deposition = smoothstep(0.10, 4.5, geology.sediment_m[index]);
    let inherited_cover = smoothstep(3.5, 12.0, tectonics.sediment_m[index]);
    let depositional_cover = active_deposition * 0.55 + inherited_cover * 0.45;
    let continental_margin = shallow_marine * (0.45 + stable * 0.55);
    let land_basin = f32::from(!ocean)
        * low_relief
        * (0.25 + hydrology.lake[index] * 0.35 + hydrology.wetness[index] * 0.20 + stable * 0.20);
    let basin_accommodation = continental_margin.max(land_basin);
    let organic_productivity = smoothstep(
        0.04,
        0.58,
        vegetation.biomass[index] + soil.organic_fraction[index] * 0.65,
    );
    let marine_organic_source =
        shallow_marine * (0.48 + (climate.wind_east[index].abs() / 14.0).clamp(0.0, 1.0) * 0.22);
    let source_rock_potential =
        marine_organic_source.max(organic_productivity * hydrology.wetness[index]);
    let oil_maturity = smoothstep(12.0, 95.0, age) * (1.0 - smoothstep(850.0, 2_400.0, age));
    let gas_maturity = smoothstep(55.0, 420.0, age);

    let bauxite_host = if !ocean
        && climate.temperature_c[index] >= 20.0
        && climate.precipitation_mm[index] >= 1_200.0
        && geology.weathering[index] >= 0.2
        && arc_or_craton > 0.0
    {
        0.22 + warm * 0.18 + humid * 0.18 + geology.weathering[index] * 0.28 + low_relief * 0.14
    } else {
        0.0
    };
    let peat_host = if soil_kind == SoilKind::Peat
        && soil.organic_fraction[index] >= 0.2
        && hydrology.wetness[index] >= 0.2
    {
        0.40 + soil.organic_fraction[index] * 0.32 + hydrology.wetness[index] * 0.28
    } else {
        0.0
    };
    let coal_host = if !ocean && organic_productivity >= 0.08 && land_basin >= 0.08 {
        (0.11
            + organic_productivity * 0.34
            + hydrology.wetness[index] * 0.18
            + depositional_cover * 0.17
            + land_basin * 0.20)
            * basin_host
    } else {
        0.0
    };
    let petroleum_host =
        if basin_accommodation >= 0.08 && source_rock_potential >= 0.06 && oil_maturity > 0.0 {
            (0.08
                + source_rock_potential * 0.30
                + basin_accommodation * 0.25
                + depositional_cover * 0.17
                + oil_maturity * 0.20)
                * basin_host
        } else {
            0.0
        };
    let gas_host =
        if basin_accommodation >= 0.08 && source_rock_potential >= 0.05 && gas_maturity > 0.0 {
            (0.08
                + source_rock_potential * 0.24
                + basin_accommodation * 0.23
                + depositional_cover * 0.21
                + gas_maturity * 0.24)
                * basin_host
        } else {
            0.0
        };
    let clay_host = if soil.clay_fraction[index] >= 0.12 && low_relief >= 0.10 {
        0.15 + soil.clay_fraction[index] * 0.42 + active_deposition * 0.23 + low_relief * 0.20
    } else {
        0.0
    };
    let upwelling = 0.48 + (climate.wind_east[index].abs() / 12.0).clamp(0.0, 1.0) * 0.32;
    let phosphate_host = shallow_marine * (0.28 + active_deposition * 0.52) * upwelling;
    let nitrate_host = if climate.aridity[index] >= 0.75
        && matches!(soil_kind, SoilKind::Desert | SoilKind::Saline)
        && hydrology.runoff[index] <= 0.5
    {
        0.30 + arid * 0.42 + (1.0 - hydrology.runoff[index]) * 0.28
    } else {
        0.0
    };

    let orogenic_gold = (tectonics.convergence[index] * 0.50
        + inherited_orogen * 0.36
        + tectonics.shear[index] * 0.08
        + tectonics.volcanism[index] * 0.06)
        * metamorphic_belt;
    let placer_gold = hydrology.river_strength[index].powf(1.35)
        * smoothstep(0.4, 24.0, hydrology.sediment_m[index])
        * (0.30
            + smoothstep(250.0, 2_600.0, tectonics.uplift_m[index].abs()) * 0.42
            + smoothstep(0.5, 75.0, hydrology.erosion_m[index]) * 0.28)
        * metamorphic_belt
        * f32::from(!ocean);

    [
        smoothstep(1_800.0, 3_200.0, age)
            * f32::from(lithology == Lithology::FelsicCraton)
            * stable,
        bauxite_host,
        tectonics.convergence[index].powf(1.15)
            * tectonics.volcanism[index]
            * subduction_arc
            * arc_or_pluton,
        tectonics.divergence[index].powf(1.3)
            * tectonics.volcanism[index]
            * extensional_margin
            * f32::from(ocean || lithology == Lithology::OceanicBasalt),
        tectonics.volcanism[index]
            * mafic_host
            * (0.35 + extensional_margin * 0.35 + tectonics.boundary[index] * 0.30),
        orogenic_gold.max(placer_gold),
        (smoothstep(400.0, 2_800.0, tectonics.uplift_m[index].abs()) * 0.55
            + tectonics.metamorphic_grade[index] * 0.30
            + tectonics.suture[index] * 0.15)
            * f32::from(matches!(
                lithology,
                Lithology::Metamorphic | Lithology::Plutonic
            ))
            * (0.45 + smoothstep(0.5, 100.0, hydrology.erosion_m[index]) * 0.55),
        coal_host,
        peat_host,
        petroleum_host,
        gas_host,
        arid * (0.42 + hydrology.lake[index] * 0.36 + shallow_marine * 0.22)
            * smoothstep(12.0, 70.0, geology.sediment_m[index])
            * (1.0 - smoothstep(0.025, 0.11, slope)),
        clay_host,
        phosphate_host,
        nitrate_host,
    ]
}

fn seed_districts(
    grid: AtlasGrid,
    seed: u64,
    deposit_index: usize,
    model: DepositModel,
    host_scores: &[[f32; DEPOSIT_COUNT]],
) -> Vec<District> {
    let spacing = scaled_reference_cells(grid, model.spacing_cells).max(2);
    let bins_x = grid.width().div_ceil(spacing);
    let bins_y = grid.height().div_ceil(spacing);
    let gate_domain = deposit_domain(DISTRICT_GATE_DOMAIN, model.family);
    let site_domain = deposit_domain(DISTRICT_SITE_DOMAIN, model.family);
    let mut candidates = Vec::new();

    for bin_y in 0..bins_y {
        for bin_x in 0..bins_x {
            let x_start = bin_x * spacing;
            let y_start = bin_y * spacing;
            let x_end = (x_start + spacing).min(grid.width());
            let y_end = (y_start + spacing).min(grid.height());
            let mut best: Option<DistrictCandidate> = None;

            for y in y_start..y_end {
                for x in x_start..x_end {
                    let index = grid.index(x, y);
                    let host_score = host_scores[index][deposit_index];
                    if host_score < model.host_floor {
                        continue;
                    }
                    let site_rank = hash_unit(seed, index, site_domain);
                    let priority = host_score * (0.78 + site_rank * 0.22);
                    if best.is_none_or(|candidate| priority > candidate.priority) {
                        let point = grid.point(index);
                        best = Some(DistrictCandidate {
                            index,
                            center: direction(point.latitude, point.longitude),
                            priority,
                            host_score,
                        });
                    }
                }
            }

            let Some(candidate) = best else {
                continue;
            };
            let bin_index = bin_y * bins_x + bin_x;
            let eligibility = smoothstep(model.host_floor, 0.85, candidate.host_score);
            let gate_probability = model.frequency * (0.50 + eligibility * 0.50);
            if hash_unit(seed, bin_index, gate_domain) <= gate_probability {
                candidates.push(candidate);
            }
        }
    }

    candidates.sort_by(|left, right| right.priority.total_cmp(&left.priority));
    let grid_height = f64::from(u32::try_from(grid.height()).expect("atlas height fits u32"));
    let minimum_spacing = (f64::from(model.spacing_cells) * std::f64::consts::PI
        / f64::from(REFERENCE_ATLAS_HEIGHT)
        * 0.36)
        .max(std::f64::consts::PI / grid_height * 0.8);
    let minimum_chord_squared = 2.0 * (1.0 - minimum_spacing.cos());
    let mut districts = Vec::with_capacity(candidates.len());

    for candidate in candidates {
        let overlaps = districts.iter().any(|district: &District| {
            2.0 * (1.0 - candidate.center.dot(district.center)) < minimum_chord_squared
        });
        if !overlaps {
            districts.push(make_district(grid, seed, model, candidate));
        }
    }

    districts
}

fn make_district(
    grid: AtlasGrid,
    seed: u64,
    model: DepositModel,
    candidate: DistrictCandidate,
) -> District {
    let point = grid.point(candidate.index);
    let (sin_lon, cos_lon) = point.longitude.sin_cos();
    let (sin_lat, cos_lat) = point.latitude.sin_cos();
    let east = Vec3::new(-sin_lon, 0.0, cos_lon);
    let north = Vec3::new(-sin_lat * cos_lon, cos_lat, -sin_lat * sin_lon);
    let axis_domain = deposit_domain(DISTRICT_AXIS_DOMAIN, model.family);
    let azimuth = f64::from(hash_unit(seed, candidate.index, axis_domain)) * std::f64::consts::TAU;
    let (sin_axis, cos_axis) = azimuth.sin_cos();
    let major_axis = Vec3::new(
        east.x * cos_axis + north.x * sin_axis,
        east.y * cos_axis + north.y * sin_axis,
        east.z * cos_axis + north.z * sin_axis,
    )
    .normalized();
    let minor_axis = candidate.center.cross(major_axis).normalized();
    let size_domain = deposit_domain(DISTRICT_SIZE_DOMAIN, model.deposit as u8);
    let size = 0.78 + f64::from(hash_unit(seed, candidate.index, size_domain)) * 0.44;
    let grid_height = f64::from(u32::try_from(grid.height()).expect("atlas height fits u32"));
    let cell_angle = std::f64::consts::PI / grid_height;
    let major_angle = (f64::from(model.major_radius_cells) * std::f64::consts::PI
        / f64::from(REFERENCE_ATLAS_HEIGHT)
        * size)
        .max(cell_angle * 1.15);
    let minor_angle = (f64::from(model.minor_radius_cells) * std::f64::consts::PI
        / f64::from(REFERENCE_ATLAS_HEIGHT)
        * size)
        .max(cell_angle * 0.82);
    let maximum_angle = major_angle.max(minor_angle) * 1.16;
    let strength_domain = deposit_domain(DISTRICT_STRENGTH_DOMAIN, model.deposit as u8);
    let strength = (0.82
        + hash_unit(seed, candidate.index, strength_domain) * 0.28
        + candidate.host_score * 0.10)
        .min(1.2);

    District {
        center: candidate.center,
        major_axis,
        minor_axis,
        major_sine: major_angle.sin().max(1.0e-6),
        minor_sine: minor_angle.sin().max(1.0e-6),
        minimum_alignment: maximum_angle.cos(),
        strength,
    }
}

#[allow(clippy::cast_possible_truncation)] // Normalized angular distances are stored as renderer-facing f32 values.
fn district_focus(position: Vec3, fabric: f32, districts: &[District]) -> f32 {
    let outline = 0.84 + (fabric - 0.5) * 0.34;
    districts
        .iter()
        .filter(|district| position.dot(district.center) >= district.minimum_alignment)
        .map(|district| {
            let along = position.dot(district.major_axis) / district.major_sine;
            let across = position.dot(district.minor_axis) / district.minor_sine;
            let elliptical_distance = (along * along + across * across).sqrt() as f32;
            (1.0 - smoothstep(0.18, outline, elliptical_distance)) * district.strength
        })
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0)
}

fn structural_fabric(grid: AtlasGrid, index: usize, seed: u64, model: DepositModel) -> f32 {
    let regional_period = (model.spacing_cells * 0.58).max(6.0);
    let local_period = (model.minor_radius_cells * 2.2).max(3.5);
    let family_domain = deposit_domain(REGIONAL_FABRIC_DOMAIN, model.family);
    let deposit_domain = deposit_domain(LOCAL_FABRIC_DOMAIN, model.deposit as u8);
    let regional = coherent_value_field(grid, index, seed, regional_period, family_domain);
    let local = coherent_value_field(grid, index, seed, local_period, deposit_domain);
    (regional * 0.64 + local * 0.36).clamp(0.0, 1.0)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::many_single_char_names
)] // Atlas dimensions are capped at 2048x1024; interpolation intentionally uses f32 storage precision.
fn coherent_value_field(
    grid: AtlasGrid,
    index: usize,
    seed: u64,
    reference_period_cells: f32,
    domain: u64,
) -> f32 {
    let period = reference_period_cells * grid.height() as f32 / REFERENCE_ATLAS_HEIGHT;
    let lattice_x = ((grid.width() as f32 / period.max(1.0)).round() as usize).max(2);
    let lattice_y = ((grid.height() as f32 / period.max(1.0)).round() as usize).max(2);
    let (x, y) = grid.coordinates(index);
    let sample_x = (x as f32 + 0.5) / grid.width() as f32 * lattice_x as f32;
    let sample_y = (y as f32 + 0.5) / grid.height() as f32 * (lattice_y - 1) as f32;
    let x0 = sample_x.floor() as usize % lattice_x;
    let y0 = (sample_y.floor() as usize).min(lattice_y - 1);
    let x1 = (x0 + 1) % lattice_x;
    let y1 = (y0 + 1).min(lattice_y - 1);
    let tx = smoothstep(0.0, 1.0, sample_x.fract());
    let ty = smoothstep(0.0, 1.0, sample_y.fract());
    let a = hash_unit(seed, y0 * lattice_x + x0, domain);
    let b = hash_unit(seed, y0 * lattice_x + x1, domain);
    let c = hash_unit(seed, y1 * lattice_x + x0, domain);
    let d = hash_unit(seed, y1 * lattice_x + x1, domain);
    let north = a + (b - a) * tx;
    let south = c + (d - c) * tx;
    north + (south - north) * ty
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)] // Atlas dimensions are capped and converted to the integer district lattice.
fn scaled_reference_cells(grid: AtlasGrid, reference_cells: f32) -> usize {
    (reference_cells * grid.height() as f32 / REFERENCE_ATLAS_HEIGHT).round() as usize
}

fn deposit_domain(base: u64, discriminator: u8) -> u64 {
    base ^ u64::from(discriminator).wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

fn deposit_depth_m(
    deposit: ResourceDeposit,
    sediment_m: f32,
    uplift_m: f32,
    coherent_variation: f32,
) -> f32 {
    let base = match deposit {
        ResourceDeposit::None => 0.0,
        ResourceDeposit::Bauxite
        | ResourceDeposit::Peat
        | ResourceDeposit::Clay
        | ResourceDeposit::Nitrate => 4.0 + sediment_m.min(20.0) * 0.25,
        ResourceDeposit::Coal | ResourceDeposit::RockSalt => 180.0 + sediment_m * 16.0,
        ResourceDeposit::Petroleum => 1_400.0 + sediment_m * 24.0,
        ResourceDeposit::NaturalGas => 2_100.0 + sediment_m * 28.0,
        ResourceDeposit::Gold | ResourceDeposit::Gemstones => {
            65.0 + uplift_m.abs().min(4_000.0) * 0.18
        }
        _ => 220.0 + sediment_m * 7.0,
    };
    (base * (0.78 + coherent_variation * 0.44)).clamp(0.0, 6_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_models_cover_every_non_empty_category_in_order() {
        assert_eq!(DEPOSIT_MODELS.len(), DEPOSIT_COUNT);
        for (index, model) in DEPOSIT_MODELS.iter().enumerate() {
            assert_eq!(usize::from(model.deposit as u8), index + 1);
            assert!(model.host_floor > 0.0);
            assert!(model.occurrence_floor > 0.0);
            assert!(model.rich_score > model.occurrence_floor);
            assert!(model.frequency > 0.0 && model.frequency <= 1.0);
        }
    }

    #[test]
    fn structural_fabric_is_longitude_periodic() {
        let grid = AtlasGrid::new(384, 192);
        let model = DEPOSIT_MODELS[0];
        let west = structural_fabric(grid, grid.index(0, 96), 42, model);
        let east = structural_fabric(grid, grid.index(383, 96), 42, model);
        assert!((west - east).abs() < 0.18);
    }
}
