//! Deterministic, engine-independent long-timescale world simulation.
//!
//! The simulation resolves coupled processes into a compact global atlas. It is
//! deliberately renderer-neutral: native tools, analysis programs, and tile
//! streamers all sample the same immutable [`WorldSnapshot`].

mod grid;
mod layers;
mod random;
mod settings;
mod snapshot;
mod stages;

pub use grid::AtlasGrid;
pub use layers::{
    Biome, ClimateSample, CrustKind, GeologySample, HydrologySample, KoppenZone, Lithology,
    ResourceDeposit, ResourcesSample, SoilKind, SoilSample, TectonicsSample, VegetationSample,
    WorldDataLayer, WorldSample,
};
pub use settings::SimulationSettings;
pub use snapshot::WorldSnapshot;

/// Common simulation contracts used by renderers and tools.
pub mod prelude {
    pub use crate::{AtlasGrid, SimulationSettings, WorldDataLayer, WorldSample, WorldSnapshot};
}
