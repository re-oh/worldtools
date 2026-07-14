import { bell, saturate } from "../../../lib/math/scalar";
import type { ConstraintSet } from "../../constraints";
import type { FieldMap } from "../../model";
import type { WorldRecipe } from "../../recipe";

export interface VegetationFields {
  vegetation: Float32Array;
  forest: Float32Array;
  grass: Float32Array;
  wetland: Float32Array;
  biome: Float32Array;
}

export function createVegetation(recipe: WorldRecipe, fields: FieldMap, constraints: ConstraintSet): VegetationFields {
  const count = fields.elevation.length;
  const vegetation = new Float32Array(count);
  const forest = new Float32Array(count);
  const grass = new Float32Array(count);
  const wetland = new Float32Array(count);
  const biome = new Float32Array(count);
  for (let cell = 0; cell < count; cell += 1) {
    if (fields.waterDepth[cell] > 0) continue;
    const thermal = bell(fields.temperature[cell], 22, 24) * saturate((fields.temperature[cell] + 12) / 18);
    const water = saturate((1 - fields.aridity[cell]) * 0.72 + fields.regolithMoisture[cell] * 0.38);
    const substrate = 0.42 + fields.mineralFertility[cell] * 0.42 + fields.organicCarbon[cell] * 1.4;
    const disturbance = saturate(fields.storminess[cell] * 0.16 + fields.channelMobility[cell] * 0.22 + fields.ice[cell] * 0.9);
    const cover = saturate(thermal * water * substrate * recipe.vegetationProductivity * (1 - disturbance) + constraints.vegetation[cell]);
    const wet = saturate((fields.floodplain[cell] * 0.58 + fields.regolithMoisture[cell] * 0.5 + saturate(fields.lakeDepth[cell] / 6) * 0.4) * cover * (1 - fields.channelMobility[cell] * 0.35));
    const treeClimate = saturate((fields.precipitation[cell] - 420) / 1250) * (1 - fields.aridity[cell]) * (1 - saturate(fields.temperatureSeasonality[cell] / 55) * 0.42);
    const treeCover = saturate(cover * treeClimate * (0.58 + fields.regolithDepth[cell] / 12) * (1 - wet * 0.35));
    const grassClimate = saturate(1 - Math.abs(fields.aridity[cell] - 0.43) / 0.5) * saturate((fields.temperature[cell] + 7) / 22);
    const grassCover = saturate((cover - treeCover * 0.72) * (0.45 + grassClimate * 0.75) * (1 - wet * 0.55));
    vegetation[cell] = cover;
    forest[cell] = treeCover;
    grass[cell] = grassCover;
    wetland[cell] = wet;
    biome[cell] = classifyBiome(fields, cell, treeCover, grassCover, wet);
  }
  return { vegetation, forest, grass, wetland, biome };
}

function classifyBiome(fields: FieldMap, cell: number, forest: number, grass: number, wetland: number): number {
  if (fields.ice[cell] > 0.52) return 1;
  if (fields.temperature[cell] < -5) return 2;
  if (fields.elevation[cell] > 3000 && fields.temperature[cell] < 8) return 11;
  if (wetland > 0.52) return 10;
  if (fields.aridity[cell] > 0.76) return 9;
  if (forest > 0.42) {
    if (fields.temperature[cell] < 7) return 3;
    if (fields.temperature[cell] < 21) return 4;
    return 5;
  }
  if (grass > 0.38) return fields.temperature[cell] > 18 && fields.aridity[cell] > 0.36 ? 7 : 6;
  if (fields.aridity[cell] > 0.48) return 8;
  return fields.temperature[cell] < 5 ? 2 : 6;
}
