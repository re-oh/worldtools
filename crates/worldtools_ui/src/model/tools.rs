/// Editor tools are intentionally semantic; render and generation plugins decide
/// how the associated interaction is performed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveTool {
    #[default]
    Navigate,
    Inspect,
}

impl ActiveTool {
    pub const ALL: [Self; 2] = [Self::Navigate, Self::Inspect];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Navigate => "Navigate",
            Self::Inspect => "Inspect",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Navigate => "Pan and zoom the map",
            Self::Inspect => "Sample generated layers",
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_navigation_and_inspection_are_exposed() {
        assert_eq!(ActiveTool::ALL, [ActiveTool::Navigate, ActiveTool::Inspect]);
        assert_eq!(ActiveTool::default(), ActiveTool::Navigate);
    }

    #[test]
    fn map_views_are_derived_from_available_elevation_data() {
        assert_eq!(MapViewMode::ALL.len(), 3);
        assert_eq!(MapViewMode::default(), MapViewMode::Terrain);
        assert_eq!(MapViewMode::Elevation.label(), "Elevation");
        assert!(!MapViewMode::Slope.description().is_empty());
    }
}
