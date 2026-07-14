use serde::{Deserialize, Serialize};

/// Stable renderer-facing world datasets. Discriminants are shader IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WorldDataLayer {
    Elevation = 0,
    Tectonics = 1,
    Hydrology = 2,
    Climate = 3,
    Soil = 4,
    Vegetation = 5,
    Geology = 6,
    Resources = 7,
}

impl WorldDataLayer {
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

    #[must_use]
    pub const fn shader_id(self) -> u32 {
        self as u32
    }

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CrustKind {
    Oceanic = 0,
    Continental = 1,
}

impl CrustKind {
    pub(crate) const fn from_byte(value: u8) -> Self {
        if value == 0 {
            Self::Oceanic
        } else {
            Self::Continental
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Oceanic => "Oceanic crust",
            Self::Continental => "Continental crust",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum KoppenZone {
    IceCap = 0,
    Tundra = 1,
    Arid = 2,
    Temperate = 3,
    Continental = 4,
    Tropical = 5,
    Ocean = 6,
}

impl KoppenZone {
    pub(crate) const fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::IceCap,
            1 => Self::Tundra,
            2 => Self::Arid,
            3 => Self::Temperate,
            4 => Self::Continental,
            5 => Self::Tropical,
            _ => Self::Ocean,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::IceCap => "Ice cap",
            Self::Tundra => "Tundra",
            Self::Arid => "Arid",
            Self::Temperate => "Temperate",
            Self::Continental => "Continental",
            Self::Tropical => "Tropical",
            Self::Ocean => "Ocean",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SoilKind {
    Ocean = 0,
    BareRock = 1,
    Cryosol = 2,
    Desert = 3,
    Chernozem = 4,
    Forest = 5,
    Laterite = 6,
    Volcanic = 7,
    Alluvial = 8,
    Peat = 9,
    Saline = 10,
}

impl SoilKind {
    pub const COUNT: f32 = 11.0;

    pub(crate) const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::BareRock,
            2 => Self::Cryosol,
            3 => Self::Desert,
            4 => Self::Chernozem,
            5 => Self::Forest,
            6 => Self::Laterite,
            7 => Self::Volcanic,
            8 => Self::Alluvial,
            9 => Self::Peat,
            10 => Self::Saline,
            _ => Self::Ocean,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ocean => "Marine sediment",
            Self::BareRock => "Bare rock",
            Self::Cryosol => "Cryosol",
            Self::Desert => "Desert soil",
            Self::Chernozem => "Chernozem",
            Self::Forest => "Forest soil",
            Self::Laterite => "Laterite",
            Self::Volcanic => "Volcanic soil",
            Self::Alluvial => "Alluvial soil",
            Self::Peat => "Peat",
            Self::Saline => "Saline soil",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Biome {
    Ocean = 0,
    Ice = 1,
    Tundra = 2,
    BorealForest = 3,
    TemperateForest = 4,
    TemperateGrassland = 5,
    Mediterranean = 6,
    Desert = 7,
    Savanna = 8,
    TropicalSeasonalForest = 9,
    TropicalRainforest = 10,
    Alpine = 11,
    Wetland = 12,
}

impl Biome {
    pub const COUNT: f32 = 13.0;

    pub(crate) const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Ice,
            2 => Self::Tundra,
            3 => Self::BorealForest,
            4 => Self::TemperateForest,
            5 => Self::TemperateGrassland,
            6 => Self::Mediterranean,
            7 => Self::Desert,
            8 => Self::Savanna,
            9 => Self::TropicalSeasonalForest,
            10 => Self::TropicalRainforest,
            11 => Self::Alpine,
            12 => Self::Wetland,
            _ => Self::Ocean,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ocean => "Ocean",
            Self::Ice => "Ice",
            Self::Tundra => "Tundra",
            Self::BorealForest => "Boreal forest",
            Self::TemperateForest => "Temperate forest",
            Self::TemperateGrassland => "Temperate grassland",
            Self::Mediterranean => "Mediterranean scrub",
            Self::Desert => "Desert",
            Self::Savanna => "Savanna",
            Self::TropicalSeasonalForest => "Tropical seasonal forest",
            Self::TropicalRainforest => "Tropical rainforest",
            Self::Alpine => "Alpine",
            Self::Wetland => "Wetland",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Lithology {
    OceanicBasalt = 0,
    FelsicCraton = 1,
    Sedimentary = 2,
    VolcanicArc = 3,
    Plutonic = 4,
    Metamorphic = 5,
    Carbonate = 6,
    Unconsolidated = 7,
}

impl Lithology {
    pub const COUNT: f32 = 8.0;

    pub(crate) const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::FelsicCraton,
            2 => Self::Sedimentary,
            3 => Self::VolcanicArc,
            4 => Self::Plutonic,
            5 => Self::Metamorphic,
            6 => Self::Carbonate,
            7 => Self::Unconsolidated,
            _ => Self::OceanicBasalt,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OceanicBasalt => "Oceanic basalt",
            Self::FelsicCraton => "Felsic craton",
            Self::Sedimentary => "Sedimentary rock",
            Self::VolcanicArc => "Volcanic arc",
            Self::Plutonic => "Plutonic rock",
            Self::Metamorphic => "Metamorphic rock",
            Self::Carbonate => "Carbonate platform",
            Self::Unconsolidated => "Unconsolidated sediment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResourceDeposit {
    None = 0,
    BandedIron = 1,
    Bauxite = 2,
    PorphyryCopper = 3,
    VolcanogenicSulfide = 4,
    Nickel = 5,
    Gold = 6,
    Gemstones = 7,
    Coal = 8,
    Peat = 9,
    Petroleum = 10,
    NaturalGas = 11,
    RockSalt = 12,
    Clay = 13,
    Phosphate = 14,
    Nitrate = 15,
}

impl ResourceDeposit {
    pub const COUNT: f32 = 16.0;

    pub(crate) const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::BandedIron,
            2 => Self::Bauxite,
            3 => Self::PorphyryCopper,
            4 => Self::VolcanogenicSulfide,
            5 => Self::Nickel,
            6 => Self::Gold,
            7 => Self::Gemstones,
            8 => Self::Coal,
            9 => Self::Peat,
            10 => Self::Petroleum,
            11 => Self::NaturalGas,
            12 => Self::RockSalt,
            13 => Self::Clay,
            14 => Self::Phosphate,
            15 => Self::Nitrate,
            _ => Self::None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "No dominant deposit",
            Self::BandedIron => "Banded iron formation",
            Self::Bauxite => "Bauxite",
            Self::PorphyryCopper => "Porphyry copper",
            Self::VolcanogenicSulfide => "Volcanogenic massive sulfide",
            Self::Nickel => "Nickel sulfide",
            Self::Gold => "Gold",
            Self::Gemstones => "Gemstones",
            Self::Coal => "Coal",
            Self::Peat => "Peat",
            Self::Petroleum => "Petroleum",
            Self::NaturalGas => "Natural gas",
            Self::RockSalt => "Rock salt",
            Self::Clay => "Clay",
            Self::Phosphate => "Phosphate",
            Self::Nitrate => "Nitrate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TectonicsSample {
    pub plate_id: u16,
    pub crust: CrustKind,
    pub crust_age_myr: f32,
    pub boundary: f32,
    pub convergence: f32,
    pub divergence: f32,
    pub volcanism: f32,
    pub uplift_m: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HydrologySample {
    pub runoff: f32,
    pub river_strength: f32,
    pub wetness: f32,
    pub lake: f32,
    pub erosion_m: f32,
    pub sediment_m: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClimateSample {
    pub zone: KoppenZone,
    pub temperature_c: f32,
    pub precipitation_mm: f32,
    pub seasonality: f32,
    pub wind_east: f32,
    pub wind_north: f32,
    pub aridity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoilSample {
    pub kind: SoilKind,
    pub depth_m: f32,
    pub fertility: f32,
    pub clay_fraction: f32,
    pub organic_fraction: f32,
    pub drainage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VegetationSample {
    pub biome: Biome,
    pub canopy_fraction: f32,
    pub grass_fraction: f32,
    pub biomass: f32,
    pub fire_frequency: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeologySample {
    pub lithology: Lithology,
    pub rock_age_myr: f32,
    pub sediment_m: f32,
    pub volcanic_ash_m: f32,
    pub weathering: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourcesSample {
    pub dominant: ResourceDeposit,
    pub richness: f32,
    pub depth_m: f32,
    pub confidence: f32,
    pub metallic: f32,
    pub energy: f32,
    pub industrial: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldSample {
    pub elevation_m: f32,
    pub slope: f32,
    pub tectonics: TectonicsSample,
    pub hydrology: HydrologySample,
    pub climate: ClimateSample,
    pub soil: SoilSample,
    pub vegetation: VegetationSample,
    pub geology: GeologySample,
    pub resources: ResourcesSample,
}

impl WorldSample {
    /// Renderer-neutral channels using physical values and raw categorical IDs.
    #[must_use]
    pub fn layer_channels(self, layer: WorldDataLayer) -> [f32; 4] {
        match layer {
            WorldDataLayer::Elevation => [self.elevation_m, self.slope, 0.0, 1.0],
            WorldDataLayer::Tectonics => [
                f32::from(self.tectonics.plate_id),
                self.tectonics.convergence - self.tectonics.divergence,
                self.tectonics.uplift_m,
                self.tectonics.volcanism,
            ],
            WorldDataLayer::Hydrology => [
                self.hydrology.river_strength,
                self.hydrology.wetness.max(self.hydrology.lake),
                self.hydrology.sediment_m,
                self.hydrology.erosion_m,
            ],
            WorldDataLayer::Climate => [
                self.climate.temperature_c,
                self.climate.precipitation_mm,
                self.climate.wind_east,
                self.climate.wind_north,
            ],
            WorldDataLayer::Soil => [
                f32::from(self.soil.kind as u8),
                self.soil.depth_m,
                self.soil.fertility,
                self.soil.organic_fraction,
            ],
            WorldDataLayer::Vegetation => [
                f32::from(self.vegetation.biome as u8),
                self.vegetation.canopy_fraction,
                self.vegetation.grass_fraction,
                self.vegetation.biomass,
            ],
            WorldDataLayer::Geology => [
                f32::from(self.geology.lithology as u8),
                self.geology.rock_age_myr,
                self.geology.sediment_m,
                self.geology.volcanic_ash_m,
            ],
            WorldDataLayer::Resources => [
                f32::from(self.resources.dominant as u8),
                self.resources.richness,
                (self.resources.depth_m / 6_000.0).clamp(0.0, 1.0),
                self.resources.confidence,
            ],
        }
    }
}
