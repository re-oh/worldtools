use serde::{Deserialize, Serialize};

/// Resolution and long-timescale controls for the global history pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationSettings {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub plate_count: u16,
    pub hotspot_count: u16,
    /// Duration represented by tectonic and weathering integration.
    pub geological_age_myr: u16,
    pub erosion_iterations: u16,
    pub moisture_iterations: u16,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            atlas_width: 384,
            atlas_height: 192,
            plate_count: 22,
            hotspot_count: 14,
            geological_age_myr: 240,
            erosion_iterations: 12,
            moisture_iterations: 24,
        }
    }
}

impl SimulationSettings {
    pub(crate) fn sanitized(self) -> Self {
        Self {
            atlas_width: self.atlas_width.clamp(32, 2_048),
            atlas_height: self.atlas_height.clamp(16, 1_024),
            plate_count: self.plate_count.clamp(4, 96),
            hotspot_count: self.hotspot_count.clamp(1, 64),
            geological_age_myr: self.geological_age_myr.clamp(10, 1_000),
            erosion_iterations: self.erosion_iterations.clamp(1, 96),
            moisture_iterations: self.moisture_iterations.clamp(4, 128),
        }
    }

    pub(crate) fn hash_into(self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.atlas_width.to_le_bytes());
        hasher.update(&self.atlas_height.to_le_bytes());
        hasher.update(&self.plate_count.to_le_bytes());
        hasher.update(&self.hotspot_count.to_le_bytes());
        hasher.update(&self.geological_age_myr.to_le_bytes());
        hasher.update(&self.erosion_iterations.to_le_bytes());
        hasher.update(&self.moisture_iterations.to_le_bytes());
    }
}
