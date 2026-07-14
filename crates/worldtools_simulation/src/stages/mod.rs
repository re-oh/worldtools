mod climate;
mod ecology;
mod geology;
mod glaciation;
mod hydrology;
mod math;
mod multires;
mod resources;
mod tectonics;

pub(crate) use climate::simulate as simulate_climate;
pub(crate) use ecology::simulate as simulate_ecology;
pub(crate) use geology::evolve_surface_geology;
pub(crate) use glaciation::simulate as simulate_glaciation;
pub(crate) use hydrology::{
    refresh_after_climate as refresh_hydrology, simulate as simulate_hydrology,
};
pub(crate) use resources::simulate as simulate_resources;
pub(crate) use tectonics::simulate as simulate_tectonics;
