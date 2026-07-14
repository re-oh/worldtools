import { bell, saturate } from "../../../../lib/math/scalar";
import type { WorldEngineContext } from "../../context";
import { burialMaturity, land, spatialProvince, tectonicStability } from "../modelHelpers";
import type { ResourceSystemModel } from "../types";

function coalFoundation(context: WorldEngineContext, cell: number): number {
  const fields = context.fields;
  const paleoWetland = spatialProvince(context, cell, 907, 3.5) * (0.35 + fields.wetlandCover[cell] * 0.35 + fields.organicCarbon[cell] * 2.5);
  const lowClasticDilution = 1 - saturate(fields.erosion[cell] / 130 + fields.channelMobility[cell] * 0.45);
  const system = fields.sedimentaryBasin[cell] * paleoWetland * lowClasticDilution * (0.45 + tectonicStability(context, cell) * 0.55);
  return land(context, cell) * Math.pow(Math.max(0, system), 0.42);
}

function petroleumSystem(context: WorldEngineContext, cell: number): number {
  const fields = context.fields;
  const source = fields.sedimentaryBasin[cell] * (0.35 + fields.clayFraction[cell] * 0.3 + fields.carbonatePlatform[cell] * 0.35) * (0.45 + spatialProvince(context, cell, 919, 4) * 0.55);
  const reservoir = saturate(fields.carbonatePlatform[cell] * 0.65 + (1 - fields.clayFraction[cell]) * fields.sedimentaryBasin[cell] * 0.55 + fields.sediment[cell] / 180);
  const seal = saturate(fields.clayDeposit[cell] * 0.62 + fields.evaporite[cell] * 0.7 + fields.clayFraction[cell] * 0.35);
  const trap = saturate(0.2 + fields.uplift[cell] * 0.28 + fields.shear[cell] * 0.18 + fields.sedimentaryBasin[cell] * tectonicStability(context, cell) * 0.48);
  const preservation = saturate(1 - Math.max(fields.convergence[cell] * 0.48, fields.erosion[cell] / 180));
  return Math.pow(Math.max(0, source * reservoir * seal * trap * preservation), 0.2);
}

export const ORGANIC_RESOURCE_MODELS: readonly ResourceSystemModel[] = [
  {
    typeId: "peat-bog", name: "Peatland", resourceClass: "organic", abundance: "organic", outputs: ["peat"], geometry: "seam",
    host: "Waterlogged organic wetland sediment", setting: "Bog, fen, backswamp, or poorly drained coastal plain", formation: "Plant production exceeded slow anaerobic decay on a persistently waterlogged, low-energy surface, allowing partially decomposed organic matter to accumulate.", commodities: ["peat"],
    minimumScore: 0.42, maximumOccurrences: 12, minimumSpacingKm: 220, radiusKm: [3, 85], ageMa: [0.0001, 0.02], depthMeters: [0, 4], thicknessMeters: [0.3, 12],
    score: (c, i) => land(c, i) * Math.pow(Math.max(0, c.fields.wetlandCover[i] * c.fields.vegetationCover[i] * c.fields.regolithMoisture[i] * (1 - c.fields.channelMobility[i] * 0.62) * (0.65 + c.fields.organicCarbon[i] * 2)), 0.45)
  },
  {
    typeId: "lignite-coal", name: "Lignite basin", resourceClass: "organic", abundance: "organic", outputs: ["coal"], geometry: "seam",
    host: "Buried peat-bearing fluvial and coastal-plain strata", setting: "Young, mildly subsided sedimentary basin", formation: "Thick peat was buried below clastic cover and compacted under low thermal maturity into brown coal while preserving abundant volatile matter.", commodities: ["lignite"],
    minimumScore: 0.32, maximumOccurrences: 8, minimumSpacingKm: 360, radiusKm: [12, 130], ageMa: [2, 180], depthMeters: [30, 900], thicknessMeters: [1, 65],
    score: (c, i) => coalFoundation(c, i) * bell(burialMaturity(c, i), 0.3, 0.22)
  },
  {
    typeId: "bituminous-coal", name: "Bituminous coalfield", resourceClass: "organic", abundance: "organic", outputs: ["coal"], geometry: "seam",
    host: "Moderately buried coal measures", setting: "Long-lived subsiding foreland, rift, or coastal basin", formation: "Peat seams underwent deeper burial, compaction, and sustained heating that expelled water and volatiles and raised carbon rank to bituminous coal.", commodities: ["thermal coal", "coking coal"],
    minimumScore: 0.32, maximumOccurrences: 8, minimumSpacingKm: 400, radiusKm: [15, 160], ageMa: [20, 420], depthMeters: [350, 2400], thicknessMeters: [1, 45],
    score: (c, i) => coalFoundation(c, i) * bell(burialMaturity(c, i), 0.56, 0.24)
  },
  {
    typeId: "anthracite-coal", name: "Anthracite district", resourceClass: "organic", abundance: "organic", outputs: ["coal"], geometry: "seam",
    host: "Deformed high-rank coal measures", setting: "Deeply buried and subsequently compressed basin margin", formation: "Earlier coal seams were driven to high rank by deep burial and tectonic heating during compression, producing low-volatile anthracite.", commodities: ["anthracite"],
    minimumScore: 0.36, maximumOccurrences: 4, minimumSpacingKm: 520, radiusKm: [8, 90], ageMa: [40, 520], depthMeters: [800, 3600], thicknessMeters: [1, 28],
    score: (c, i) => coalFoundation(c, i) * saturate((burialMaturity(c, i) - 0.58) / 0.35) * saturate(c.fields.convergence[i] * 0.5 + c.fields.uplift[i] * 0.35 + c.fields.shear[i] * 0.25)
  },
  {
    typeId: "conventional-oil", name: "Conventional oil province", resourceClass: "hydrocarbon", abundance: "organic", outputs: ["hydrocarbon"], geometry: "reservoir",
    host: "Porous sandstone or carbonate beneath a regional seal", setting: "Mature sedimentary basin with source, carrier, reservoir, trap, and seal", formation: "Buried organic-rich source rock entered the oil window, generated petroleum, and expelled it through carrier beds into a sealed structural or stratigraphic trap.", commodities: ["light crude oil", "associated gas", "natural gas liquids"],
    minimumScore: 0.36, maximumOccurrences: 8, minimumSpacingKm: 500, radiusKm: [12, 110], ageMa: [10, 420], depthMeters: [600, 4200], thicknessMeters: [8, 180],
    score: (c, i) => land(c, i) * petroleumSystem(c, i) * bell(burialMaturity(c, i), 0.54, 0.22)
  },
  {
    typeId: "heavy-oil", name: "Heavy-oil accumulation", resourceClass: "hydrocarbon", abundance: "organic", outputs: ["hydrocarbon"], geometry: "reservoir",
    host: "Shallow porous reservoir near a basin margin", setting: "Uplifted or shallow petroleum trap exposed to biodegradation and volatile loss", formation: "Migrated oil accumulated at shallow depth where cooling, water washing, biodegradation, and loss of light fractions increased viscosity and density.", commodities: ["heavy crude oil", "bitumen"],
    minimumScore: 0.35, maximumOccurrences: 5, minimumSpacingKm: 480, radiusKm: [10, 95], ageMa: [5, 320], depthMeters: [40, 900], thicknessMeters: [10, 160],
    score: (c, i) => land(c, i) * petroleumSystem(c, i) * bell(burialMaturity(c, i), 0.4, 0.24) * saturate(c.fields.weathering[i] * 0.4 + c.fields.uplift[i] * 0.35 + 0.3)
  },
  {
    typeId: "thermogenic-gas", name: "Thermogenic gas province", resourceClass: "hydrocarbon", abundance: "organic", outputs: ["hydrocarbon"], geometry: "reservoir",
    host: "Deep porous reservoir beneath an effective seal", setting: "Deep mature to overmature sedimentary basin", formation: "Sustained burial heated source organic matter and pre-existing oil into the gas window; buoyant gas migrated into sealed reservoirs.", commodities: ["methane", "wet gas", "natural gas liquids"],
    minimumScore: 0.36, maximumOccurrences: 7, minimumSpacingKm: 520, radiusKm: [14, 125], ageMa: [20, 520], depthMeters: [1400, 6500], thicknessMeters: [12, 240],
    score: (c, i) => land(c, i) * petroleumSystem(c, i) * bell(burialMaturity(c, i), 0.78, 0.22)
  },
  {
    typeId: "biogenic-gas", name: "Biogenic gas field", resourceClass: "hydrocarbon", abundance: "organic", outputs: ["hydrocarbon"], geometry: "reservoir",
    host: "Shallow fine-grained organic basin sediment", setting: "Cool young basin or delta with rapid burial and microbial methanogenesis", formation: "Anaerobic microbes converted shallow buried organic matter to methane before significant thermal maturation, which accumulated beneath local seals.", commodities: ["methane"],
    minimumScore: 0.32, maximumOccurrences: 7, minimumSpacingKm: 360, radiusKm: [6, 80], ageMa: [0.1, 80], depthMeters: [80, 1200], thicknessMeters: [8, 130],
    score: (c, i) => land(c, i) * petroleumSystem(c, i) * bell(burialMaturity(c, i), 0.25, 0.2) * saturate(c.fields.organicCarbon[i] * 4 + c.fields.wetlandCover[i] * 0.45)
  }
];
