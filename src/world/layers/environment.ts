import { PALETTES as P } from "./palettes";
import { defineLayer } from "./types";

export const ENVIRONMENT_LAYERS = [
  defineLayer("elevation", "Relief", "Terrain", "terrain", "m", "continuous", P.terrain, { fixedRange: [-8000, 7000], hillshade: true }),
  defineLayer("slope", "Slope", "Terrain", "hydrology", "rise/run", "log", P.earth),
  defineLayer("erosion", "Erosion", "Terrain", "hydrology", "m", "log", P.earth),
  defineLayer("sediment", "Deposited sediment", "Terrain", "hydrology", "m", "log", P.earth),
  defineLayer("waterDepth", "Ocean depth", "Ocean", "terrain", "m", "log", P.water),
  defineLayer("oceanCurrent", "Current speed", "Ocean", "ocean", "index", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("upwelling", "Upwelling", "Ocean", "ocean", "index", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("seaSurfaceTemperature", "Sea surface temperature", "Ocean", "ocean", "C", "continuous", P.heat, { fixedRange: [-3, 34] }),
  defineLayer("temperature", "Mean temperature", "Climate", "climate", "C", "continuous", P.heat, { fixedRange: [-45, 40] }),
  defineLayer("temperatureSeasonality", "Temperature seasonality", "Climate", "climate", "C", "continuous", P.heat, { fixedRange: [0, 50] }),
  defineLayer("precipitation", "Precipitation", "Climate", "climate", "mm/yr", "log", P.water),
  defineLayer("aridity", "Aridity", "Climate", "climate", "index", "continuous", [[0, "#397981"], [0.5, "#8c8b59"], [1, "#d7a65c"]], { fixedRange: [0, 1] }),
  defineLayer("windSpeed", "Prevailing wind", "Climate", "climate", "index", "continuous", P.stress, { fixedRange: [0, 1] }),
  defineLayer("storminess", "Storm exposure", "Climate", "climate", "index", "continuous", P.stress, { fixedRange: [0, 1] }),
  defineLayer("freezeThaw", "Freeze-thaw", "Climate", "climate", "index", "continuous", P.ice, { fixedRange: [0, 1] }),
  defineLayer("lakeDepth", "Lake depth", "Hydrology", "hydrology", "m", "log", P.water),
  defineLayer("flow", "River discharge", "Hydrology", "hydrology", "index", "log", P.water, { fixedRange: [0, 1] }),
  defineLayer("runoff", "Runoff", "Hydrology", "hydrology", "mm/yr", "log", P.water),
  defineLayer("floodplain", "Floodplain", "Hydrology", "hydrology", "index", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("channelMobility", "Channel mobility", "Hydrology", "hydrology", "index", "continuous", P.stress, { fixedRange: [0, 1] }),
  defineLayer("delta", "Delta deposition", "Hydrology", "hydrology", "index", "continuous", P.earth, { fixedRange: [0, 1] }),
  defineLayer("ice", "Persistent ice", "Cryosphere", "cryosphere", "index", "continuous", P.ice, { fixedRange: [0, 1] }),
  defineLayer("glacialErosion", "Glacial erosion", "Cryosphere", "cryosphere", "m", "log", P.ice),
  defineLayer("glacialDeposit", "Till and outwash", "Cryosphere", "cryosphere", "m", "log", P.earth),
  defineLayer("weathering", "Chemical weathering", "Soils", "soils", "index", "continuous", P.soil, { fixedRange: [0, 1] }),
  defineLayer("regolithDepth", "Regolith depth", "Soils", "soils", "m", "continuous", P.soil, { fixedRange: [0, 8] }),
  defineLayer("regolithMoisture", "Soil moisture", "Soils", "soils", "index", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("clayFraction", "Clay fraction", "Soils", "soils", "fraction", "continuous", P.soil, { fixedRange: [0, 1] }),
  defineLayer("organicCarbon", "Organic carbon", "Soils", "soils", "fraction", "continuous", P.organic, { fixedRange: [0, 0.2] }),
  defineLayer("soilPH", "Soil pH", "Soils", "soils", "pH", "continuous", P.stress, { fixedRange: [3.5, 9.5] }),
  defineLayer("soilType", "Soil order", "Soils", "soils", "class", "categorical", P.soil, {
    categories: { 0: "Ocean / none", 1: "Cryosol", 2: "Histosol", 3: "Podzol", 4: "Temperate forest soil", 5: "Mollisol", 6: "Oxisol", 7: "Aridisol", 8: "Vertisol", 9: "Fluvisol", 10: "Andisol" }
  }),
  defineLayer("mineralFertility", "Mineral fertility", "Soils", "soils", "index", "continuous", P.vegetation, { fixedRange: [0, 1] }),
  defineLayer("texture", "Fine texture", "Soils", "soils", "sand-clay", "diverging", P.soil, { fixedRange: [0, 1] }),
  defineLayer("vegetationCover", "Vegetation cover", "Ecology", "ecology", "fraction", "continuous", P.vegetation, { fixedRange: [0, 1] }),
  defineLayer("forestCover", "Forest cover", "Ecology", "ecology", "fraction", "continuous", P.vegetation, { fixedRange: [0, 1] }),
  defineLayer("grassCover", "Grass and plains", "Ecology", "ecology", "fraction", "continuous", P.vegetation, { fixedRange: [0, 1] }),
  defineLayer("wetlandCover", "Wetland cover", "Ecology", "ecology", "fraction", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("biome", "Biome", "Ecology", "ecology", "class", "categorical", P.vegetation, {
    categories: { 0: "Ocean", 1: "Ice", 2: "Tundra", 3: "Boreal forest", 4: "Temperate forest", 5: "Tropical forest", 6: "Grassland", 7: "Savanna", 8: "Shrubland", 9: "Desert", 10: "Wetland", 11: "Alpine" }
  })
] as const;
