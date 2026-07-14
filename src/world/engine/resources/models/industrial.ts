import { bell, saturate } from "../../../../lib/math/scalar";
import type { ResourceSystemModel } from "../types";
import { ageWindow, land, lithology, ocean, spatialProvince, tectonicStability, tropicalWeathering } from "../modelHelpers";

export const INDUSTRIAL_RESOURCE_MODELS: readonly ResourceSystemModel[] = [
  {
    typeId: "residual-kaolin", name: "Residual kaolin district", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["clayDeposit"], geometry: "residual-blanket",
    host: "Deeply weathered feldspathic crystalline rock", setting: "Warm humid stable upland with limited physical erosion", formation: "Acidic meteoric water leached feldspar-rich parent rock in place, leaving a thick kaolinite-dominant saprolite.", commodities: ["kaolin", "halloysite"],
    minimumScore: 0.48, maximumOccurrences: 8, minimumSpacingKm: 330, radiusKm: [4, 60], ageMa: [1, 180], depthMeters: [0, 25], thicknessMeters: [4, 75],
    score: (c, i) => land(c, i) * tropicalWeathering(c, i) * (0.48 + lithology(c, i, 2, 4) * 0.52) * tectonicStability(c, i) * (0.55 + c.fields.clayFraction[i] * 0.45)
  },
  {
    typeId: "bentonite", name: "Bentonite bed district", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["clayDeposit"], geometry: "stratiform",
    host: "Altered volcanic-ash beds in a quiet sedimentary basin", setting: "Lake, delta, shelf, or restricted marine basin downwind of explosive volcanism", formation: "Fine volcanic ash settled into standing water and altered diagenetically to smectite-rich clay while protected from coarse clastic dilution.", commodities: ["bentonite", "smectite"],
    minimumScore: 0.46, maximumOccurrences: 7, minimumSpacingKm: 300, radiusKm: [8, 95], ageMa: [1, 260], depthMeters: [0, 850], thicknessMeters: [1, 24],
    score: (c, i) => c.fields.volcanism[i] * (0.35 + c.fields.sedimentaryBasin[i] * 0.65) * (1 - saturate(c.fields.slope[i] * 85)) * (0.45 + spatialProvince(c, i, 811, 5) * 0.55)
  },
  {
    typeId: "sedimentary-clay", name: "Sedimentary clay basin", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["clayDeposit"], geometry: "stratiform",
    host: "Fine-grained alluvial, deltaic, lacustrine, or shelf sediment", setting: "Low-energy depositional basin", formation: "Weathered clay minerals and fine rock flour were transported, hydraulically sorted, and concentrated where river, lake, delta, or shelf energy declined.", commodities: ["ball clay", "fire clay", "common clay"],
    minimumScore: 0.47, maximumOccurrences: 10, minimumSpacingKm: 260, radiusKm: [10, 130], ageMa: [0.01, 180], depthMeters: [0, 420], thicknessMeters: [3, 90],
    score: (c, i) => land(c, i) * c.fields.clayFraction[i] * saturate(c.fields.floodplain[i] * 0.45 + c.fields.delta[i] * 0.8 + c.fields.sedimentaryBasin[i] * 0.45 + c.fields.sediment[i] / 150) * (1 - saturate(c.fields.slope[i] * 100))
  },
  {
    typeId: "marine-evaporite", name: "Marine evaporite basin", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["evaporite"], geometry: "stratiform",
    host: "Restricted shallow-marine carbonate and mudstone basin", setting: "Arid subsiding shelf or half-graben with episodic seawater recharge", formation: "Evaporation repeatedly exceeded inflow in a restricted basin, precipitating gypsum, then halite and locally potash salts as brine concentration increased.", commodities: ["gypsum", "halite", "potash", "magnesium salts"],
    minimumScore: 0.46, maximumOccurrences: 7, minimumSpacingKm: 480, radiusKm: [30, 240], ageMa: [2, 520], depthMeters: [0, 2400], thicknessMeters: [20, 1200],
    score: (c, i) => ocean(c, i) * bell(c.fields.waterDepth[i], 130, 240) * saturate((c.fields.aridity[i] - 0.25) / 0.65) * (0.42 + c.fields.sedimentaryBasin[i] * 0.58) * (1 - c.fields.oceanCurrent[i] * 0.62)
  },
  {
    typeId: "playa-evaporite", name: "Continental playa salts", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["evaporite"], geometry: "playa-crust",
    host: "Closed-basin playa mud and saline lake sediment", setting: "Arid interior drainage basin", formation: "Intermittent runoff delivered dissolved ions to a closed depression where repeated lake expansion and desiccation built layered carbonate, sulfate, and chloride salts.", commodities: ["halite", "gypsum", "borates", "lithium brine"],
    minimumScore: 0.49, maximumOccurrences: 7, minimumSpacingKm: 360, radiusKm: [8, 120], ageMa: [0.01, 80], depthMeters: [0, 160], thicknessMeters: [2, 190],
    score: (c, i) => land(c, i) * saturate((c.fields.aridity[i] - 0.48) / 0.48) * saturate(c.fields.lakeDepth[i] / 14 + c.fields.sedimentaryBasin[i] * 0.45) * (1 - saturate(c.fields.slope[i] * 100)) * spatialProvince(c, i, 827, 4)
  },
  {
    typeId: "nitrate-caliche", name: "Nitrate caliche field", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["nitrate"], geometry: "playa-crust",
    host: "Old hyperarid piedmont, playa margin, and caliche regolith", setting: "Long-lived rain-shadow desert with negligible leaching", formation: "Atmospheric and locally volcanic nitrogen accumulated over immense time because rainfall, biological cycling, erosion, and groundwater flushing were exceptionally weak.", commodities: ["sodium nitrate", "potassium nitrate", "iodate"],
    minimumScore: 0.62, maximumOccurrences: 3, minimumSpacingKm: 850, radiusKm: [6, 85], ageMa: [1, 40], depthMeters: [0, 6], thicknessMeters: [0.2, 5],
    score: (c, i) => land(c, i) * saturate((c.fields.aridity[i] - 0.76) / 0.22) * saturate((180 - c.fields.precipitation[i]) / 160) * tectonicStability(c, i) * (1 - c.fields.weathering[i]) * (0.38 + c.fields.sedimentaryBasin[i] * 0.32 + c.fields.volcanism[i] * 0.3) * spatialProvince(c, i, 839, 7)
  },
  {
    typeId: "pegmatite-gem", name: "Gem pegmatite field", resourceClass: "gemstone", abundance: "mineral", outputs: ["gemstone"], geometry: "pegmatite-dike",
    host: "Evolved granitic pegmatite dikes and pockets", setting: "Old continental crust near a felsic plutonic or rift complex", formation: "Water- and flux-rich residual granitic melt crystallized slowly in coarse dikes, concentrating rare elements and growing large gem-quality crystals in open pockets.", commodities: ["beryl", "tourmaline", "topaz", "spodumene", "quartz"],
    minimumScore: 0.54, maximumOccurrences: 6, minimumSpacingKm: 430, radiusKm: [1, 22], ageMa: [10, 2200], depthMeters: [0, 2600], thicknessMeters: [1, 40],
    score: (c, i) => land(c, i) * lithology(c, i, 2) * ageWindow(c, i, 1400, 1500) * saturate(c.fields.uplift[i] * 0.35 + c.fields.divergence[i] * 0.35 + c.fields.volcanism[i] * 0.3) * spatialProvince(c, i, 853, 8)
  },
  {
    typeId: "metamorphic-gem", name: "Metamorphic gemstone belt", resourceClass: "gemstone", abundance: "mineral", outputs: ["gemstone"], geometry: "vein-corridor",
    host: "High-grade metamorphic and metasomatic rock", setting: "Exhumed collisional or subduction-related metamorphic belt", formation: "Pressure, temperature, deformation, and reactive fluids recrystallized suitable aluminum-, chromium-, or beryllium-bearing protoliths into localized gem-bearing zones.", commodities: ["corundum", "garnet", "jade", "spinel", "beryl"],
    minimumScore: 0.52, maximumOccurrences: 6, minimumSpacingKm: 390, radiusKm: [2, 36], ageMa: [10, 2600], depthMeters: [0, 3800], thicknessMeters: [2, 85],
    score: (c, i) => land(c, i) * (0.35 + lithology(c, i, 3, 5) * 0.65) * saturate(c.fields.convergence[i] * 0.45 + c.fields.uplift[i] * 0.38 + c.fields.shear[i] * 0.28) * spatialProvince(c, i, 859, 6)
  }
];
