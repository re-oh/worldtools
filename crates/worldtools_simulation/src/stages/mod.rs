mod climate;
mod ecology;
mod geology;
mod hydrology;
mod math;
mod resources;
mod tectonics;

pub(crate) use climate::simulate as simulate_climate;
pub(crate) use ecology::simulate as simulate_ecology;
pub(crate) use geology::evolve_surface_geology;
pub(crate) use hydrology::{
    refresh_after_climate as refresh_hydrology, simulate as simulate_hydrology,
};
pub(crate) use resources::simulate as simulate_resources;
pub(crate) use tectonics::simulate as simulate_tectonics;
