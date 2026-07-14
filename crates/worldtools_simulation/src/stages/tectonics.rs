use rayon::prelude::*;
use worldtools_world::{TerrainGenerator, TerrainSettings, WorldSeed};

use crate::{
    AtlasGrid, SimulationSettings,
    layers::BoundaryKind,
    random::StableRng,
    stages::math::{Vec3, direction, smoothstep},
};

#[derive(Debug)]
struct Plate {
    center: Vec3,
    angular_motion: Vec3,
    continental: bool,
    age_myr: f32,
}

#[derive(Debug)]
struct Hotspot {
    center: Vec3,
    reach_cosine: f64,
    strength: f32,
}

#[derive(Debug)]
struct Cell {
    baseline_elevation_m: f32,
    elevation_m: f32,
    plate_id: u16,
    paleo_plate_id: u16,
    terrane_id: u16,
    crust: u8,
    boundary_kind: u8,
    crust_age_myr: f32,
    crust_thickness_km: f32,
    boundary: f32,
    convergence: f32,
    divergence: f32,
    shear: f32,
    suture: f32,
    metamorphic_grade: f32,
    volcanism: f32,
    uplift_m: f32,
    lithology: u8,
    rock_age_myr: f32,
    sediment_m: f32,
    volcanic_ash_m: f32,
}

#[derive(Debug)]
pub(crate) struct TectonicState {
    pub(crate) baseline_elevation_m: Vec<f32>,
    pub(crate) elevation_m: Vec<f32>,
    pub(crate) plate_id: Vec<u16>,
    pub(crate) paleo_plate_id: Vec<u16>,
    pub(crate) terrane_id: Vec<u16>,
    pub(crate) crust: Vec<u8>,
    pub(crate) boundary_kind: Vec<u8>,
    pub(crate) crust_age_myr: Vec<f32>,
    pub(crate) crust_thickness_km: Vec<f32>,
    pub(crate) boundary: Vec<f32>,
    pub(crate) convergence: Vec<f32>,
    pub(crate) divergence: Vec<f32>,
    pub(crate) shear: Vec<f32>,
    pub(crate) suture: Vec<f32>,
    pub(crate) metamorphic_grade: Vec<f32>,
    pub(crate) volcanism: Vec<f32>,
    pub(crate) uplift_m: Vec<f32>,
    pub(crate) lithology: Vec<u8>,
    pub(crate) rock_age_myr: Vec<f32>,
    pub(crate) sediment_m: Vec<f32>,
    pub(crate) volcanic_ash_m: Vec<f32>,
}

#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
pub(crate) fn simulate(
    grid: AtlasGrid,
    seed: WorldSeed,
    terrain: TerrainSettings,
    settings: SimulationSettings,
) -> TectonicState {
    let plates = make_plates(seed, settings);
    let paleo_centers = paleo_plate_centers(&plates, settings);
    let hotspots = make_hotspots(seed, settings);
    let terrain_generator = TerrainGenerator::new(seed, terrain);
    let boundary_scale = f64::from(settings.plate_count).sqrt() * 8.0;
    let history_scale = (f32::from(settings.geological_age_myr) / 240.0)
        .sqrt()
        .clamp(0.35, 1.8);

    let cells = (0..grid.len())
        .into_par_iter()
        .map(|index| {
            let point = grid.point(index);
            let position = direction(point.latitude, point.longitude);
            let mut nearest = 0_usize;
            let mut nearest_dot = f64::NEG_INFINITY;
            let mut second = 0_usize;
            let mut second_dot = f64::NEG_INFINITY;

            for (plate_index, plate) in plates.iter().enumerate() {
                let similarity = position.dot(plate.center);
                if similarity > nearest_dot {
                    second = nearest;
                    second_dot = nearest_dot;
                    nearest = plate_index;
                    nearest_dot = similarity;
                } else if similarity > second_dot {
                    second = plate_index;
                    second_dot = similarity;
                }
            }

            let (paleo_nearest, _paleo_second, paleo_gap) = nearest_two(position, &paleo_centers);
            let paleo_boundary = (-paleo_gap * boundary_scale).exp() as f32;

            let plate = &plates[nearest];
            let neighbor = &plates[second];
            let boundary = (-(nearest_dot - second_dot) * boundary_scale).exp() as f32;
            let toward_neighbor = neighbor
                .center
                .sub(position.scale(position.dot(neighbor.center)))
                .normalized();
            let primary_velocity = plate.angular_motion.cross(position);
            let neighbor_velocity = neighbor.angular_motion.cross(position);
            let separation = neighbor_velocity
                .sub(primary_velocity)
                .dot(toward_neighbor)
                .clamp(-0.16, 0.16)
                / 0.16;
            let convergence = boundary * (-separation).max(0.0) as f32;
            let divergence = boundary * separation.max(0.0) as f32;
            let tangent = position.cross(toward_neighbor).normalized();
            let shear = boundary
                * (neighbor_velocity.sub(primary_velocity).dot(tangent).abs() / 0.16)
                    .clamp(0.0, 1.0) as f32;
            let suture = (paleo_boundary * (0.35 + (1.0 - boundary) * 0.65)).clamp(0.0, 1.0);
            let boundary_kind = classify_boundary(
                boundary,
                convergence,
                divergence,
                shear,
                plate.continental,
                neighbor.continental,
            );
            let ocean_involved = f32::from(!plate.continental || !neighbor.continental);
            let boundary_volcanism = convergence * ocean_involved * 0.9
                + divergence * 0.55
                + f32::from(matches!(boundary_kind, BoundaryKind::SubductionArc))
                    * convergence
                    * 0.18;
            let hotspot_volcanism = hotspots
                .iter()
                .map(|hotspot| hotspot_activity(position, hotspot, plate.angular_motion))
                .fold(0.0_f32, f32::max);
            let volcanism = (boundary_volcanism + hotspot_volcanism).clamp(0.0, 1.0);

            let base = terrain_generator.sample_geo(point);
            let plate_crust_bias = if plate.continental { 420.0 } else { -260.0 };
            let continental_weight = smoothstep(
                -1_050.0,
                420.0,
                base - terrain.sea_level_m + plate_crust_bias,
            );
            let continental_crust = continental_weight >= 0.5;
            let buoyancy = -180.0 + continental_weight * 430.0;
            let collision_relief = match boundary_kind {
                BoundaryKind::ContinentalCollision => 4_700.0,
                BoundaryKind::SubductionArc => 3_300.0,
                BoundaryKind::IslandArc => 2_650.0,
                _ => 2_100.0 + continental_weight * 1_400.0,
            };
            let collision_uplift = convergence.powf(1.35) * collision_relief * history_scale;
            let spreading_relief = 1_250.0 - continental_weight * 1_700.0;
            let spreading_ridge = divergence.powf(1.2) * spreading_relief;
            let volcanic_construction = smoothstep(0.28, 0.92, volcanism).powf(2.1)
                * if base < terrain.sea_level_m {
                    5_200.0
                } else {
                    1_350.0
                }
                * history_scale;
            let inherited_relief = suture.powf(1.7) * continental_weight * 760.0 * history_scale;
            let transform_relief = shear.powf(1.5) * 380.0;
            let uplift_m = buoyancy
                + collision_uplift
                + spreading_ridge
                + volcanic_construction
                + inherited_relief
                + transform_relief;
            let elevation_m = base + uplift_m;
            let crust_age_myr = if continental_crust {
                plate.age_myr
            } else {
                (plate.age_myr * (1.0 - divergence * 0.85)).clamp(0.0, 240.0)
            };
            let metamorphic_grade =
                (convergence * 0.72 + suture * 0.42 + shear * 0.12).clamp(0.0, 1.0);
            let crust_thickness_km = (if continental_crust { 34.0 } else { 7.0 }
                + convergence * if continental_crust { 22.0 } else { 8.0 }
                + suture * if continental_crust { 9.0 } else { 2.0 })
            .clamp(5.0, 72.0);
            let lithology = if volcanism > 0.62 {
                3
            } else if metamorphic_grade > 0.62 {
                5
            } else if continental_crust && crust_age_myr > 1_800.0 {
                1
            } else if continental_crust {
                4
            } else {
                0
            };

            Cell {
                baseline_elevation_m: base,
                elevation_m,
                plate_id: u16::try_from(nearest).expect("plate count fits u16"),
                paleo_plate_id: u16::try_from(paleo_nearest).expect("plate count fits u16"),
                terrane_id: u16::try_from(nearest * plates.len() + paleo_nearest)
                    .expect("plate history signature fits u16"),
                crust: u8::from(continental_crust),
                boundary_kind: boundary_kind as u8,
                crust_age_myr,
                crust_thickness_km,
                boundary,
                convergence,
                divergence,
                shear,
                suture,
                metamorphic_grade,
                volcanism,
                uplift_m,
                lithology,
                rock_age_myr: if volcanism > 0.5 {
                    crust_age_myr.min(25.0) * (1.0 - volcanism)
                } else {
                    crust_age_myr
                },
                sediment_m: if elevation_m < terrain.sea_level_m {
                    8.0
                } else {
                    1.0
                },
                volcanic_ash_m: volcanism.powi(3) * 4.0,
            }
        })
        .collect::<Vec<_>>();

    unpack(cells)
}

fn make_plates(seed: WorldSeed, settings: SimulationSettings) -> Vec<Plate> {
    let mut rng = StableRng::new(seed.key("simulation.plates.v1").u64());
    (0..settings.plate_count)
        .map(|_| {
            let z = f64::from(rng.signed_f32());
            let theta = f64::from(rng.unit_f32()) * std::f64::consts::TAU;
            let radial = (1.0 - z * z).sqrt();
            let center = Vec3::new(radial * theta.cos(), z, radial * theta.sin());
            let motion_axis = Vec3::new(
                f64::from(rng.signed_f32()),
                f64::from(rng.signed_f32()),
                f64::from(rng.signed_f32()),
            )
            .normalized();
            let speed = 0.035 + f64::from(rng.unit_f32()) * 0.09;
            let continental = rng.unit_f32() < 0.43;
            let age_myr = if continental {
                450.0 + rng.unit_f32().powf(0.55) * 3_450.0
            } else {
                8.0 + rng.unit_f32() * 210.0
            };
            Plate {
                center,
                angular_motion: motion_axis.scale(speed),
                continental,
                age_myr,
            }
        })
        .collect()
}

fn make_hotspots(seed: WorldSeed, settings: SimulationSettings) -> Vec<Hotspot> {
    let mut rng = StableRng::new(seed.key("simulation.hotspots.v1").u64());
    (0..settings.hotspot_count)
        .map(|_| {
            let z = f64::from(rng.signed_f32());
            let theta = f64::from(rng.unit_f32()) * std::f64::consts::TAU;
            let radial = (1.0 - z * z).sqrt();
            let radius_degrees = 1.25 + f64::from(rng.unit_f32()) * 2.25;
            Hotspot {
                center: Vec3::new(radial * theta.cos(), z, radial * theta.sin()),
                reach_cosine: radius_degrees.to_radians().cos(),
                strength: 0.55 + rng.unit_f32() * 0.45,
            }
        })
        .collect()
}

fn paleo_plate_centers(plates: &[Plate], settings: SimulationSettings) -> Vec<Vec3> {
    let age_factor = (f64::from(settings.geological_age_myr) / 60.0).clamp(0.75, 6.0);
    plates
        .iter()
        .map(|plate| {
            let speed = plate.angular_motion.dot(plate.angular_motion).sqrt();
            plate.center.rotate_about(
                plate.angular_motion,
                -(speed * age_factor).clamp(0.06, 0.68),
            )
        })
        .collect()
}

fn nearest_two(position: Vec3, centers: &[Vec3]) -> (usize, usize, f64) {
    let mut nearest = (0_usize, f64::NEG_INFINITY);
    let mut second = (0_usize, f64::NEG_INFINITY);
    for (index, &center) in centers.iter().enumerate() {
        let similarity = position.dot(center);
        if similarity > nearest.1 {
            second = nearest;
            nearest = (index, similarity);
        } else if similarity > second.1 {
            second = (index, similarity);
        }
    }
    (nearest.0, second.0, nearest.1 - second.1)
}

fn classify_boundary(
    boundary: f32,
    convergence: f32,
    divergence: f32,
    shear: f32,
    primary_continental: bool,
    neighbor_continental: bool,
) -> BoundaryKind {
    if boundary < 0.16 {
        return BoundaryKind::Intraplate;
    }
    if convergence >= divergence && convergence >= shear * 0.72 {
        return match (primary_continental, neighbor_continental) {
            (true, true) => BoundaryKind::ContinentalCollision,
            (false, false) => BoundaryKind::IslandArc,
            _ => BoundaryKind::SubductionArc,
        };
    }
    if divergence >= shear * 0.8 {
        if primary_continental || neighbor_continental {
            BoundaryKind::ContinentalRift
        } else {
            BoundaryKind::OceanRidge
        }
    } else {
        BoundaryKind::Transform
    }
}

#[allow(clippy::cast_possible_truncation)] // Normalized spherical proximity is stored in the f32 simulation atlas.
fn hotspot_activity(position: Vec3, hotspot: &Hotspot, angular_motion: Vec3) -> f32 {
    let track = angular_motion.cross(hotspot.center).normalized();
    (0_u8..5)
        .map(|step| {
            let offset_radians = f64::from(step) * 0.018;
            let center = hotspot.center.sub(track.scale(offset_radians)).normalized();
            let proximity = ((position.dot(center) - hotspot.reach_cosine)
                / (1.0 - hotspot.reach_cosine))
                .clamp(0.0, 1.0) as f32;
            let age_decay = 1.0 - f32::from(step) * 0.13;
            smoothstep(0.16, 1.0, proximity).powf(1.6) * hotspot.strength * age_decay
        })
        .fold(0.0_f32, f32::max)
}

fn unpack(cells: Vec<Cell>) -> TectonicState {
    let count = cells.len();
    let mut state = TectonicState {
        baseline_elevation_m: Vec::with_capacity(count),
        elevation_m: Vec::with_capacity(count),
        plate_id: Vec::with_capacity(count),
        paleo_plate_id: Vec::with_capacity(count),
        terrane_id: Vec::with_capacity(count),
        crust: Vec::with_capacity(count),
        boundary_kind: Vec::with_capacity(count),
        crust_age_myr: Vec::with_capacity(count),
        crust_thickness_km: Vec::with_capacity(count),
        boundary: Vec::with_capacity(count),
        convergence: Vec::with_capacity(count),
        divergence: Vec::with_capacity(count),
        shear: Vec::with_capacity(count),
        suture: Vec::with_capacity(count),
        metamorphic_grade: Vec::with_capacity(count),
        volcanism: Vec::with_capacity(count),
        uplift_m: Vec::with_capacity(count),
        lithology: Vec::with_capacity(count),
        rock_age_myr: Vec::with_capacity(count),
        sediment_m: Vec::with_capacity(count),
        volcanic_ash_m: Vec::with_capacity(count),
    };
    for cell in cells {
        state.baseline_elevation_m.push(cell.baseline_elevation_m);
        state.elevation_m.push(cell.elevation_m);
        state.plate_id.push(cell.plate_id);
        state.paleo_plate_id.push(cell.paleo_plate_id);
        state.terrane_id.push(cell.terrane_id);
        state.crust.push(cell.crust);
        state.boundary_kind.push(cell.boundary_kind);
        state.crust_age_myr.push(cell.crust_age_myr);
        state.crust_thickness_km.push(cell.crust_thickness_km);
        state.boundary.push(cell.boundary);
        state.convergence.push(cell.convergence);
        state.divergence.push(cell.divergence);
        state.shear.push(cell.shear);
        state.suture.push(cell.suture);
        state.metamorphic_grade.push(cell.metamorphic_grade);
        state.volcanism.push(cell.volcanism);
        state.uplift_m.push(cell.uplift_m);
        state.lithology.push(cell.lithology);
        state.rock_age_myr.push(cell.rock_age_myr);
        state.sediment_m.push(cell.sediment_m);
        state.volcanic_ash_m.push(cell.volcanic_ash_m);
    }
    state
}
