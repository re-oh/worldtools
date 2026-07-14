use bevy::prelude::Resource;

/// Runtime controls for visual diagnostics produced by the map renderer.
///
/// These settings only affect presentation and tracing. They never change
/// generated world data or cache keys.
#[allow(clippy::struct_excessive_bools)] // Independent runtime switches, not mutually exclusive states.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct RenderDebugSettings {
    /// Draw page boundaries at their exact projected edges.
    pub tile_borders: bool,
    /// Tint each requested LOD with a stable, contrasting colour.
    pub lod_tint: bool,
    /// Tint exact, fallback, and stale pages differently.
    pub residency_tint: bool,
    /// Width of page borders in physical pixels.
    pub border_width_px: f32,
    /// Emit detailed tile lifecycle events at the `debug` tracing level.
    pub trace_streaming: bool,
    /// Pause new page requests while allowing already-running work to finish.
    pub freeze_streaming: bool,
}

impl Default for RenderDebugSettings {
    fn default() -> Self {
        Self {
            tile_borders: false,
            lod_tint: false,
            residency_tint: false,
            border_width_px: 1.0,
            trace_streaming: false,
            freeze_streaming: false,
        }
    }
}

impl RenderDebugSettings {
    pub(crate) fn shader_flags(self, stale: bool) -> u32 {
        u32::from(self.tile_borders)
            | (u32::from(self.lod_tint) << 1)
            | (u32::from(self.residency_tint) << 2)
            | (u32::from(stale) << 3)
    }
}

/// Per-frame facts about the pages submitted to the GPU.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub struct TileRenderStats {
    pub rendered: usize,
    pub exact: usize,
    pub fallback: usize,
    pub stale: usize,
    pub missing: usize,
    pub gpu_resident: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_shader_flags_are_independent() {
        let settings = RenderDebugSettings {
            tile_borders: true,
            lod_tint: false,
            residency_tint: true,
            border_width_px: 2.0,
            trace_streaming: false,
            freeze_streaming: false,
        };

        assert_eq!(settings.shader_flags(false), 0b0101);
        assert_eq!(settings.shader_flags(true), 0b1101);
    }

    #[test]
    fn diagnostics_are_opt_in() {
        let settings = RenderDebugSettings::default();
        assert!(!settings.tile_borders);
        assert!(!settings.lod_tint);
        assert!(!settings.residency_tint);
        assert!(!settings.trace_streaming);
        assert!(!settings.freeze_streaming);
    }
}
