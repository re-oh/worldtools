import type { FieldStats } from "../lib/field/field";
import { WORLD_VERSION, type WorldRecipe } from "./recipe";

export const FIELD_IDS = [
  "plateId",
  "continentalness",
  "crustAge",
  "crustThickness",
  "convergence",
  "divergence",
  "shear",
  "uplift",
  "volcanism",
  "sedimentaryBasin",
  "carbonatePlatform",
  "lithology",
  "elevation",
  "waterDepth",
  "lakeDepth",
  "slope",
  "erosion",
  "sediment",
  "oceanCurrent",
  "upwelling",
  "seaSurfaceTemperature",
  "flow",
  "runoff",
  "floodplain",
  "channelMobility",
  "delta",
  "ice",
  "glacialErosion",
  "glacialDeposit",
  "temperature",
  "temperatureSeasonality",
  "precipitation",
  "aridity",
  "windSpeed",
  "storminess",
  "freezeThaw",
  "weathering",
  "regolithDepth",
  "regolithMoisture",
  "clayFraction",
  "organicCarbon",
  "soilPH",
  "soilType",
  "mineralFertility",
  "texture",
  "vegetationCover",
  "forestCover",
  "grassCover",
  "wetlandCover",
  "biome",
  "bandedIron",
  "bauxite",
  "copperSulfide",
  "magmaticSulfide",
  "carbonatite",
  "orogenicGold",
  "copper",
  "gold",
  "iron",
  "rareEarth",
  "nickel",
  "placer",
  "mineralPotential",
  "clayDeposit",
  "peat",
  "coal",
  "hydrocarbon",
  "evaporite",
  "nitrate",
  "gemstone",
  "resourcePotential"
] as const;

export type FieldId = (typeof FIELD_IDS)[number];
export type FieldMap = Record<FieldId, Float32Array>;

export const DEPOSIT_TYPE_IDS = [
  "superior-bif",
  "algoma-bif",
  "laterite-bauxite",
  "karst-bauxite",
  "porphyry-cu",
  "cyprus-vms",
  "kuroko-vms",
  "besshi-vms",
  "komatiitic-ni-cu",
  "lateritic-ni",
  "carbonatite-ree",
  "orogenic-au",
  "alluvial-placer-au",
  "residual-kaolin",
  "bentonite",
  "sedimentary-clay",
  "peat-bog",
  "lignite-coal",
  "bituminous-coal",
  "anthracite-coal",
  "conventional-oil",
  "heavy-oil",
  "thermogenic-gas",
  "biogenic-gas",
  "marine-evaporite",
  "playa-evaporite",
  "nitrate-caliche",
  "pegmatite-gem",
  "metamorphic-gem",
  "placer-gem"
] as const;

export type DepositTypeId = (typeof DEPOSIT_TYPE_IDS)[number];

export type ResourceClass = "metal" | "industrial-mineral" | "organic" | "hydrocarbon" | "gemstone";

export interface NaturalResourceDeposit {
  id: string;
  typeId: DepositTypeId;
  resourceClass: ResourceClass;
  name: string;
  cell: number;
  radiusKm: number;
  ageMa: number;
  potential: number;
  depthMeters: number;
  thicknessMeters: number;
  quality: number;
  geometry:
    | "stratiform"
    | "stockwork"
    | "massive-lens"
    | "flow-base"
    | "residual-blanket"
    | "karst-pocket"
    | "vein-corridor"
    | "alluvial-trap"
    | "intrusive-complex"
    | "seam"
    | "reservoir"
    | "diapir"
    | "playa-crust"
    | "pegmatite-dike";
  host: string;
  setting: string;
  formation: string;
  commodities: readonly string[];
}

export const STAGE_IDS = ["geology", "tectonics", "terrain", "ocean", "climate", "cryosphere", "hydrology", "soils", "ecology", "resources"] as const;
export type WorldStageId = (typeof STAGE_IDS)[number];

export const STAGE_LABELS: Record<WorldStageId, string> = {
  geology: "Crust and plates",
  tectonics: "Tectonic history",
  terrain: "Lithosphere and relief",
  ocean: "Ocean circulation",
  climate: "Climate and weather regimes",
  cryosphere: "Ice and glacial landforms",
  hydrology: "Rivers and sediment",
  soils: "Soils and regolith",
  ecology: "Vegetation and biomes",
  resources: "Natural resource systems"
};

export interface WorldStageStatus {
  id: WorldStageId;
  label: string;
  state: "pending" | "running" | "complete" | "stale" | "failed" | "cancelled";
  progress: number;
  durationMs: number;
  detail: string;
}

export interface PlateModel {
  id: number;
  seedCell: number;
  continentalBias: number;
  ageMa: number;
  eulerPole: readonly [number, number, number];
  angularVelocityDegreesMa: number;
  growthBias: number;
  growthDirection: readonly [number, number, number];
}

export interface PackedWorld {
  version: typeof WORLD_VERSION;
  id: string;
  name: string;
  recipe: WorldRecipe;
  generatedAt: string;
  generationMs: number;
  fields: FieldMap;
  stats: Record<FieldId, FieldStats>;
  downstream: Int32Array;
  basin: Int32Array;
  plates: PlateModel[];
  deposits: NaturalResourceDeposit[];
  stages: WorldStageStatus[];
  checksum: string;
  notes: string[];
}

export interface CellSnapshot {
  index: number;
  column: number;
  row: number;
  longitudeDegrees: number;
  latitudeDegrees: number;
  values: Record<FieldId, number>;
}

export function createEmptyFieldMap(cellCount: number): FieldMap {
  const fields = {} as FieldMap;
  for (const id of FIELD_IDS) {
    Object.defineProperty(fields, id, {
      configurable: true,
      enumerable: true,
      get() {
        const values = new Float32Array(cellCount);
        installField(fields, id, values);
        return values;
      },
      set(values: Float32Array) {
        if (!(values instanceof Float32Array) || values.length !== cellCount) {
          throw new Error(`Field ${id} must be a Float32Array with ${cellCount} cells.`);
        }
        installField(fields, id, values);
      }
    });
  }
  return fields;
}

function installField(fields: FieldMap, id: FieldId, values: Float32Array): void {
  Object.defineProperty(fields, id, {
    configurable: true,
    enumerable: true,
    writable: true,
    value: values
  });
}

export function worldTransferables(world: PackedWorld): Transferable[] {
  return [...FIELD_IDS.map((id) => world.fields[id].buffer), world.downstream.buffer, world.basin.buffer];
}

export function snapshotCell(world: PackedWorld, index: number): CellSnapshot {
  const bounded = Math.max(0, Math.min(world.recipe.width * world.recipe.height - 1, index));
  const row = Math.floor(bounded / world.recipe.width);
  const column = bounded % world.recipe.width;
  const sinLatitude = -1 + ((row + 0.5) / world.recipe.height) * 2;
  return {
    index: bounded,
    column,
    row,
    longitudeDegrees: -180 + ((column + 0.5) / world.recipe.width) * 360,
    latitudeDegrees: (Math.asin(sinLatitude) / Math.PI) * 180,
    values: Object.fromEntries(FIELD_IDS.map((id) => [id, world.fields[id][bounded]])) as Record<FieldId, number>
  };
}
