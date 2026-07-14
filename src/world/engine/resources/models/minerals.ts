import { bell, saturate } from "../../../../lib/math/scalar";
import type { ResourceSystemModel } from "../types";
import { ageWindow, land, lithology, ocean, spatialProvince, tectonicStability, tropicalWeathering } from "../modelHelpers";

export const BEDROCK_MINERAL_MODELS: readonly ResourceSystemModel[] = [
  {
    typeId: "superior-bif", name: "Superior-type BIF district", resourceClass: "metal", abundance: "mineral", outputs: ["bandedIron", "iron"], geometry: "stratiform",
    host: "Chert-carbonate shelf succession", setting: "Stable Paleoproterozoic shelf or intracratonic basin", formation: "Repeated chemical precipitation of silica and iron in a low-clastic shallow-marine basin, followed by burial and possible supergene enrichment.", commodities: ["iron"],
    minimumScore: 0.5, maximumOccurrences: 6, minimumSpacingKm: 700, radiusKm: [35, 180], ageMa: [1800, 2500], depthMeters: [0, 900], thicknessMeters: [20, 450],
    score: (c, i) => ageWindow(c, i, 2150, 700) * c.fields.carbonatePlatform[i] * (0.35 + c.fields.sedimentaryBasin[i] * 0.65) * tectonicStability(c, i)
  },
  {
    typeId: "algoma-bif", name: "Algoma-type BIF belt", resourceClass: "metal", abundance: "mineral", outputs: ["bandedIron", "iron"], geometry: "stratiform",
    host: "Mafic volcanic and volcaniclastic greenstone succession", setting: "Archean submarine volcanic basin", formation: "Hydrothermal iron and silica entered a restricted deep-water volcanic basin and precipitated during quiet intervals between eruptions.", commodities: ["iron"],
    minimumScore: 0.48, maximumOccurrences: 5, minimumSpacingKm: 600, radiusKm: [18, 110], ageMa: [2400, 3600], depthMeters: [0, 1200], thicknessMeters: [8, 180],
    score: (c, i) => ageWindow(c, i, 3000, 950) * (0.35 + lithology(c, i, 1, 4) * 0.65) * c.fields.volcanism[i] * (0.35 + c.fields.sedimentaryBasin[i] * 0.65)
  },
  {
    typeId: "laterite-bauxite", name: "Lateritic bauxite plateau", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["bauxite"], geometry: "residual-blanket",
    host: (c, i) => Math.round(c.fields.lithology[i]) === 2 ? "Felsic crystalline parent" : "Aluminous mafic to sedimentary parent", setting: "Stable, well-drained tropical upland", formation: "Prolonged warm, wet leaching removed mobile silica and bases while aluminum oxides accumulated on a low-erosion surface.", commodities: ["bauxite", "aluminum"],
    minimumScore: 0.34, maximumOccurrences: 8, minimumSpacingKm: 450, radiusKm: [8, 75], ageMa: [1, 180], depthMeters: [0, 18], thicknessMeters: [2, 45],
    score: (c, i) => land(c, i) * tropicalWeathering(c, i) * bell(c.fields.elevation[i], 750, 950) * (0.55 + spatialProvince(c, i, 701, 4) * 0.45)
  },
  {
    typeId: "karst-bauxite", name: "Karst bauxite pocket", resourceClass: "industrial-mineral", abundance: "mineral", outputs: ["bauxite"], geometry: "karst-pocket",
    host: "Karstified carbonate platform", setting: "Humid carbonate terrain receiving aluminous residual sediment", formation: "Aluminous weathering residue was transported short distances and concentrated in dissolution depressions on carbonate rock.", commodities: ["bauxite", "aluminum"],
    minimumScore: 0.34, maximumOccurrences: 6, minimumSpacingKm: 350, radiusKm: [2, 28], ageMa: [1, 220], depthMeters: [0, 65], thicknessMeters: [3, 55],
    score: (c, i) => land(c, i) * tropicalWeathering(c, i) * c.fields.carbonatePlatform[i] * (0.5 + c.fields.sediment[i] / 200)
  },
  {
    typeId: "porphyry-cu", name: "Porphyry copper district", resourceClass: "metal", abundance: "mineral", outputs: ["copperSulfide", "copper"], geometry: "stockwork",
    host: "Altered felsic to intermediate porphyry", setting: "High-level convergent-margin magmatic arc", formation: "Oxidized hydrous arc magma exsolved metal-bearing fluids that fractured and altered the intrusion and its wall rock into a zoned stockwork.", commodities: ["copper", "molybdenum", "gold"],
    minimumScore: 0.54, maximumOccurrences: 8, minimumSpacingKm: 420, radiusKm: [3, 35], ageMa: [1, 320], depthMeters: [150, 3200], thicknessMeters: [250, 2100],
    score: (c, i) => land(c, i) * c.fields.convergence[i] * c.fields.volcanism[i] * (0.45 + c.fields.uplift[i] * 0.55) * (0.48 + lithology(c, i, 2, 3) * 0.52)
  },
  {
    typeId: "cyprus-vms", name: "Cyprus-type VMS camp", resourceClass: "metal", abundance: "mineral", outputs: ["copperSulfide", "copper"], geometry: "massive-lens",
    host: "Pillow basalt and sheeted dike complex", setting: "Oceanic or back-arc spreading ridge", formation: "Seawater circulated through hot oceanic crust and discharged at submarine vents, precipitating massive sulfide lenses over a stockwork feeder.", commodities: ["copper", "zinc", "sulfur"],
    minimumScore: 0.5, maximumOccurrences: 7, minimumSpacingKm: 330, radiusKm: [1, 18], ageMa: [1, 260], depthMeters: [0, 1800], thicknessMeters: [4, 90],
    score: (c, i) => ocean(c, i) * c.fields.divergence[i] * c.fields.volcanism[i] * (0.55 + lithology(c, i, 1, 5) * 0.45) * saturate(1 - c.fields.crustAge[i] / 240)
  },
  {
    typeId: "kuroko-vms", name: "Kuroko-type VMS camp", resourceClass: "metal", abundance: "mineral", outputs: ["copperSulfide", "copper"], geometry: "massive-lens",
    host: "Felsic submarine volcanic pile with mudstone", setting: "Extensional island arc or back-arc basin", formation: "Fault-focused hydrothermal discharge into an anoxic felsic volcanic basin built stacked zinc-copper sulfide lenses and feeder stringers.", commodities: ["copper", "zinc", "lead", "silver", "gold"],
    minimumScore: 0.5, maximumOccurrences: 6, minimumSpacingKm: 340, radiusKm: [1, 16], ageMa: [1, 420], depthMeters: [0, 2200], thicknessMeters: [5, 120],
    score: (c, i) => c.fields.volcanism[i] * c.fields.sedimentaryBasin[i] * (0.42 + c.fields.divergence[i] * 0.3 + c.fields.convergence[i] * 0.28) * (0.5 + lithology(c, i, 2, 4) * 0.5)
  },
  {
    typeId: "besshi-vms", name: "Besshi-type sulfide belt", resourceClass: "metal", abundance: "mineral", outputs: ["copperSulfide", "copper"], geometry: "stratiform",
    host: "Mafic tuff interbedded with deep-water clastic sediment", setting: "Anoxic rifted arc slope or back-arc basin", formation: "Hydrothermal sulfides accumulated as thin sheets in a sediment-starved mafic basin and were later deformed and metamorphosed.", commodities: ["copper", "zinc", "cobalt"],
    minimumScore: 0.48, maximumOccurrences: 5, minimumSpacingKm: 380, radiusKm: [4, 45], ageMa: [20, 1200], depthMeters: [0, 2800], thicknessMeters: [2, 65],
    score: (c, i) => c.fields.sedimentaryBasin[i] * (0.45 + lithology(c, i, 1, 3, 4) * 0.55) * saturate(c.fields.shear[i] * 0.55 + c.fields.convergence[i] * 0.45)
  },
  {
    typeId: "komatiitic-ni-cu", name: "Komatiitic nickel-copper belt", resourceClass: "metal", abundance: "mineral", outputs: ["magmaticSulfide", "nickel", "copper"], geometry: "flow-base",
    host: "Komatiitic ultramafic flow channel or feeder", setting: "Archean to Paleoproterozoic greenstone belt", formation: "Hot turbulent ultramafic magma assimilated sulfur and concentrated immiscible nickel-copper sulfide liquid at flow bases and feeder irregularities.", commodities: ["nickel", "copper", "cobalt", "platinum-group elements"],
    minimumScore: 0.52, maximumOccurrences: 5, minimumSpacingKm: 500, radiusKm: [2, 30], ageMa: [1800, 3600], depthMeters: [0, 1900], thicknessMeters: [4, 110],
    score: (c, i) => ageWindow(c, i, 2900, 1000) * lithology(c, i, 1, 5) * (0.45 + c.fields.volcanism[i] * 0.55) * spatialProvince(c, i, 733, 6)
  },
  {
    typeId: "lateritic-ni", name: "Nickel laterite profile", resourceClass: "metal", abundance: "mineral", outputs: ["nickel"], geometry: "residual-blanket",
    host: "Weathered peridotite, dunite, or serpentinite", setting: "Warm humid exposed ophiolite on a stable erosional surface", formation: "Intense chemical weathering leached an ultramafic protolith and partitioned nickel into limonite and underlying saprolite horizons.", commodities: ["nickel", "cobalt"],
    minimumScore: 0.53, maximumOccurrences: 6, minimumSpacingKm: 380, radiusKm: [4, 55], ageMa: [1, 120], depthMeters: [0, 35], thicknessMeters: [5, 55],
    score: (c, i) => land(c, i) * lithology(c, i, 5) * tropicalWeathering(c, i) * (0.55 + c.fields.convergence[i] * 0.45)
  },
  {
    typeId: "carbonatite-ree", name: "Carbonatite rare-earth complex", resourceClass: "metal", abundance: "mineral", outputs: ["carbonatite", "rareEarth"], geometry: "intrusive-complex",
    host: "Carbonatite and alkaline intrusive complex", setting: "Intraplate rift or deep lithospheric fault in old continental crust", formation: "Small-volume carbonate-rich mantle melts rose along a deep fault, fractionated, and concentrated rare earth, niobium, phosphate, and iron minerals.", commodities: ["rare-earth elements", "niobium", "phosphate", "iron"],
    minimumScore: 0.57, maximumOccurrences: 3, minimumSpacingKm: 900, radiusKm: [2, 38], ageMa: [10, 1800], depthMeters: [0, 2400], thicknessMeters: [180, 2600],
    score: (c, i) => land(c, i) * saturate(c.fields.continentalness[i] * 2) * ageWindow(c, i, 1700, 1500) * (0.25 + c.fields.divergence[i] * 0.5 + c.fields.volcanism[i] * 0.25) * spatialProvince(c, i, 751, 8)
  },
  {
    typeId: "orogenic-au", name: "Orogenic gold corridor", resourceClass: "metal", abundance: "mineral", outputs: ["orogenicGold", "gold"], geometry: "vein-corridor",
    host: "Metamorphic volcanic-sedimentary belt cut by regional faults", setting: "Compressional accreted margin or reactivated greenstone belt", formation: "Metamorphic fluids focused through long-lived shear zones, depositing quartz-carbonate veins during uplift after peak metamorphism.", commodities: ["gold", "silver"],
    minimumScore: 0.5, maximumOccurrences: 9, minimumSpacingKm: 380, radiusKm: [3, 70], ageMa: [20, 2800], depthMeters: [0, 5200], thicknessMeters: [1, 45],
    score: (c, i) => land(c, i) * saturate(c.fields.shear[i] * 0.48 + c.fields.convergence[i] * 0.34 + c.fields.uplift[i] * 0.38) * (0.45 + lithology(c, i, 1, 3) * 0.55) * spatialProvince(c, i, 769, 5)
  }
];
