import { clamp, saturate } from "../../../lib/math/scalar";
import type { FieldMap } from "../../model";
import type { WorldRecipe } from "../../recipe";

export interface SoilFields {
  weathering: Float32Array;
  regolithDepth: Float32Array;
  regolithMoisture: Float32Array;
  clayFraction: Float32Array;
  organicCarbon: Float32Array;
  soilPH: Float32Array;
  soilType: Float32Array;
  mineralFertility: Float32Array;
  texture: Float32Array;
}

export function createSoilProfiles(recipe: WorldRecipe, fields: FieldMap): SoilFields {
  const count = fields.elevation.length;
  const result: SoilFields = {
    weathering: new Float32Array(count),
    regolithDepth: new Float32Array(count),
    regolithMoisture: new Float32Array(count),
    clayFraction: new Float32Array(count),
    organicCarbon: new Float32Array(count),
    soilPH: new Float32Array(count),
    soilType: new Float32Array(count),
    mineralFertility: new Float32Array(count),
    texture: new Float32Array(count)
  };
  for (let cell = 0; cell < count; cell += 1) {
    if (fields.waterDepth[cell] > 0) continue;
    const lithology = Math.round(fields.lithology[cell]);
    const warm = saturate((fields.temperature[cell] + 5) / 32);
    const moisture = saturate((1 - fields.aridity[cell]) * fields.precipitation[cell] / 1300);
    const stable = 1 - saturate(fields.erosion[cell] / 140 + fields.slope[cell] * 90);
    const parentReactivity = lithology === 1 || lithology === 5 ? 1 : lithology === 2 ? 0.64 : lithology === 4 ? 0.84 : 0.75;
    const chemical = saturate(warm * moisture * parentReactivity * stable * recipe.soilMaturity);
    const physical = saturate(fields.freezeThaw[cell] * 0.7 + fields.glacialDeposit[cell] / 100 + fields.storminess[cell] * 0.12);
    const depositional = saturate(fields.sediment[cell] / 95 + fields.floodplain[cell] * 0.45 + fields.delta[cell] * 0.6);
    const depth = clamp(chemical * 4.8 + physical * 1.5 + depositional * 2.4 - fields.erosion[cell] / 95, 0.05, 9);
    const soilMoisture = saturate(moisture * 0.66 + fields.floodplain[cell] * 0.28 + saturate(fields.lakeDepth[cell] / 8) * 0.32 - fields.slope[cell] * 12);
    const inheritedClay = lithology === 4 ? 0.44 : lithology === 2 ? 0.22 : lithology === 5 ? 0.12 : 0.26;
    const neoformedClay = chemical * (lithology === 1 || lithology === 5 ? 0.45 : 0.32);
    const depositedClay = depositional * (1 - saturate(fields.slope[cell] * 80)) * 0.42;
    const alteredAsh = fields.volcanism[cell] * fields.sedimentaryBasin[cell] * moisture * 0.34;
    const clay = saturate(inheritedClay + neoformedClay + depositedClay + alteredAsh - physical * 0.12);
    const productivityProxy = warm * (1 - fields.aridity[cell]) * saturate(fields.precipitation[cell] / 900);
    const decomposition = saturate((fields.temperature[cell] + 2) / 30) * (0.35 + (1 - soilMoisture) * 0.65);
    const organic = clamp(productivityProxy * (1 - decomposition * 0.72) * (0.035 + soilMoisture * 0.055), 0.002, 0.18);
    const carbonate = fields.carbonatePlatform[cell];
    const ph = clamp(7.1 + carbonate * 1.25 + fields.aridity[cell] * 0.9 + Number(lithology === 1 || lithology === 5) * 0.35 - chemical * 1.55 - organic * 2.4, 3.6, 9.4);
    const parentNutrients = lithology === 1 || lithology === 5 ? 0.82 : lithology === 4 ? 0.58 : lithology === 2 ? 0.38 : 0.55;
    const fertility = saturate(parentNutrients * 0.48 + fields.volcanism[cell] * 0.2 + depositional * 0.28 + organic * 2.2 - chemical * moisture * 0.28);

    result.weathering[cell] = chemical;
    result.regolithDepth[cell] = depth;
    result.regolithMoisture[cell] = soilMoisture;
    result.clayFraction[cell] = clay;
    result.organicCarbon[cell] = organic;
    result.soilPH[cell] = ph;
    result.mineralFertility[cell] = fertility;
    result.texture[cell] = saturate(clay * 0.75 + depositional * 0.2);
    result.soilType[cell] = classifySoil(fields, cell, chemical, soilMoisture, clay, organic, ph);
  }
  return result;
}

function classifySoil(fields: FieldMap, cell: number, weathering: number, moisture: number, clay: number, organic: number, ph: number): number {
  if (fields.ice[cell] > 0.48) return 1;
  if (moisture > 0.82 && organic > 0.065 && fields.slope[cell] < 0.015) return 2;
  if (fields.volcanism[cell] > 0.55 && fields.glacialDeposit[cell] < 20) return 10;
  if (fields.floodplain[cell] > 0.48 || fields.delta[cell] > 0.42) return 9;
  if (fields.aridity[cell] > 0.67) return 7;
  if (clay > 0.62 && fields.temperatureSeasonality[cell] > 12) return 8;
  if (weathering > 0.72 && fields.temperature[cell] > 18) return 6;
  if (fields.temperature[cell] < 7 && moisture > 0.55 && ph < 5.8) return 3;
  if (fields.aridity[cell] > 0.28 && fields.aridity[cell] < 0.62 && fields.temperature[cell] > 2) return 5;
  return 4;
}
