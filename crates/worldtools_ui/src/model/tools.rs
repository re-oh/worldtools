/// Editor tools are intentionally semantic; render and generation plugins decide
/// how the associated interaction is performed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveTool {
    #[default]
    Navigate,
    Inspect,
    Sculpt,
    Smooth,
    RiverGuide,
    PaintConstraint,
}

impl ActiveTool {
    /// Tools with a complete native interaction path.
    pub const PRIMARY: [Self; 3] = [Self::Navigate, Self::Inspect, Self::Sculpt];

    /// All planned tools, including those not yet available in the editor.
    pub const ALL: [Self; 6] = [
        Self::Navigate,
        Self::Inspect,
        Self::Sculpt,
        Self::Smooth,
        Self::RiverGuide,
        Self::PaintConstraint,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Navigate => "Navigate",
            Self::Inspect => "Inspect",
            Self::Sculpt => "Sculpt terrain",
            Self::Smooth => "Smooth terrain",
            Self::RiverGuide => "Guide rivers",
            Self::PaintConstraint => "Paint constraint",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Navigate => "Pan and zoom the map",
            Self::Inspect => "Sample generated layers",
            Self::Sculpt => "Raise, lower, or flatten elevation",
            Self::Smooth => "Reduce local elevation roughness",
            Self::RiverGuide => "Bias drainage through a painted corridor",
            Self::PaintConstraint => "Paint a generator constraint mask",
        }
    }

    #[must_use]
    pub const fn uses_brush(self) -> bool {
        matches!(
            self,
            Self::Sculpt | Self::Smooth | Self::RiverGuide | Self::PaintConstraint
        )
    }

    #[must_use]
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Navigate | Self::Inspect | Self::Sculpt)
    }
}

/// Selects how the active world data is presented in the map viewport.
///
/// These modes are intentionally limited to views derivable from elevation.
/// Adding a mode backed by another dataset requires that dataset to be native.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapViewMode {
    #[default]
    Terrain,
    Elevation,
    Slope,
}

impl MapViewMode {
    pub const ALL: [Self; 3] = [Self::Terrain, Self::Elevation, Self::Slope];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Terrain => "Terrain",
            Self::Elevation => "Elevation",
            Self::Slope => "Slope",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Terrain => "Natural relief and bathymetry",
            Self::Elevation => "Absolute height above sea level",
            Self::Slope => "Terrain steepness derived from elevation",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldLayer {
    #[default]
    Elevation,
    Tectonics,
    Hydrology,
    Climate,
    Soil,
    Vegetation,
    Geology,
    Resources,
}

impl WorldLayer {
    pub const ALL: [Self; 8] = [
        Self::Elevation,
        Self::Tectonics,
        Self::Hydrology,
        Self::Climate,
        Self::Soil,
        Self::Vegetation,
        Self::Geology,
        Self::Resources,
    ];
    pub const COUNT: usize = Self::ALL.len();

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Elevation => "Elevation",
            Self::Tectonics => "Tectonics",
            Self::Hydrology => "Hydrology",
            Self::Climate => "Climate",
            Self::Soil => "Soil",
            Self::Vegetation => "Vegetation",
            Self::Geology => "Geology",
            Self::Resources => "Resources",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Elevation => "Evolved surface relief and bathymetry",
            Self::Tectonics => "Plates, boundaries, stress, uplift, and volcanism",
            Self::Hydrology => "Drainage, rivers, lakes, runoff, and sediment",
            Self::Climate => "Temperature, precipitation, aridity, and prevailing wind",
            Self::Soil => "Soil family, depth, texture, fertility, and organic content",
            Self::Vegetation => "Biome, forest and grass cover, and productivity",
            Self::Geology => "Bedrock family, crustal age, sediment, and volcanic ash",
            Self::Resources => "Process-based mineral, fuel, salt, clay, and gemstone deposits",
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BrushOperation {
    #[default]
    Raise,
    Lower,
    Flatten,
    Replace,
}

impl BrushOperation {
    pub const ALL: [Self; 4] = [Self::Raise, Self::Lower, Self::Flatten, Self::Replace];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Raise => "Raise",
            Self::Lower => "Lower",
            Self::Flatten => "Flatten",
            Self::Replace => "Replace",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BrushFalloff {
    Hard,
    Linear,
    #[default]
    Smooth,
    Gaussian,
}

impl BrushFalloff {
    pub const ALL: [Self; 4] = [Self::Hard, Self::Linear, Self::Smooth, Self::Gaussian];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hard => "Hard",
            Self::Linear => "Linear",
            Self::Smooth => "Smooth",
            Self::Gaussian => "Gaussian",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushSettings {
    pub radius_m: f32,
    pub strength: f32,
    pub spacing: f32,
    pub operation: BrushOperation,
    pub falloff: BrushFalloff,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            radius_m: 12_000.0,
            strength: 0.35,
            spacing: 0.2,
            operation: BrushOperation::Raise,
            falloff: BrushFalloff::Smooth,
        }
    }
}

impl BrushSettings {
    pub fn sanitize(&mut self) {
        self.radius_m = self.radius_m.clamp(1.0, 2_000_000.0);
        self.strength = self.strength.clamp(0.0, 1.0);
        self.spacing = self.spacing.clamp(0.02, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_values_are_sanitized_at_the_public_boundary() {
        let mut brush = BrushSettings {
            radius_m: -4.0,
            strength: 4.0,
            spacing: 0.0,
            ..Default::default()
        };
        brush.sanitize();

        assert!((brush.radius_m - 1.0).abs() < f32::EPSILON);
        assert!((brush.strength - 1.0).abs() < f32::EPSILON);
        assert!((brush.spacing - 0.02).abs() < f32::EPSILON);
    }

    #[test]
    fn primary_tools_only_expose_working_interactions() {
        assert_eq!(
            ActiveTool::PRIMARY,
            [
                ActiveTool::Navigate,
                ActiveTool::Inspect,
                ActiveTool::Sculpt
            ]
        );
        assert!(
            ActiveTool::PRIMARY
                .into_iter()
                .all(ActiveTool::is_available)
        );
        assert!(!ActiveTool::Smooth.is_available());
        assert!(!ActiveTool::RiverGuide.is_available());
        assert!(!ActiveTool::PaintConstraint.is_available());
    }

    #[test]
    fn map_views_are_derived_from_available_elevation_data() {
        assert_eq!(MapViewMode::ALL.len(), 3);
        assert_eq!(MapViewMode::default(), MapViewMode::Terrain);
        assert_eq!(MapViewMode::Elevation.label(), "Elevation");
        assert!(!MapViewMode::Slope.description().is_empty());
    }
}
