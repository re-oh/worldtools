import { bell, saturate } from "../../../lib/math/scalar";
import { fractalNoise3 } from "../../../lib/random/deterministic";
import type { WorldEngineContext } from "../context";

export function lithology(context: WorldEngineContext, cell: number, ...classes: number[]): number {
  return classes.includes(Math.round(context.fields.lithology[cell])) ? 1 : 0;
}

export function land(context: WorldEngineContext, cell: number): number {
  return context.fields.waterDepth[cell] === 0 ? 1 : 0;
}

export function ocean(context: WorldEngineContext, cell: number): number {
  return context.fields.waterDepth[cell] > 0 ? 1 : 0;
}

export function tectonicStability(context: WorldEngineContext, cell: number): number {
  const fields = context.fields;
  return saturate(1 - Math.max(fields.convergence[cell], fields.divergence[cell], fields.shear[cell], fields.volcanism[cell] * 0.7));
}

export function tropicalWeathering(context: WorldEngineContext, cell: number): number {
  const fields = context.fields;
  const warm = saturate((fields.temperature[cell] - 12) / 16);
  const wet = saturate((fields.precipitation[cell] - 700) / 1500) * (1 - fields.aridity[cell]);
  const preserved = 1 - saturate(fields.erosion[cell] / 120 + fields.slope[cell] * 85);
  return saturate(Math.pow(Math.max(0, warm * wet * fields.weathering[cell] * preserved), 0.48));
}

export function spatialProvince(context: WorldEngineContext, cell: number, stream: number, scale = 5): number {
  const grid = context.grid;
  return saturate((fractalNoise3(grid.x[cell], grid.y[cell], grid.z[cell], context.recipe.seed + stream, scale, 3, 0.52) + 1) * 0.5);
}

export function ageWindow(context: WorldEngineContext, cell: number, centerMa: number, widthMa: number): number {
  return bell(context.fields.crustAge[cell], centerMa, widthMa) * saturate((context.recipe.worldAgeGa * 1000 - centerMa + widthMa) / widthMa);
}

export function burialMaturity(context: WorldEngineContext, cell: number): number {
  const fields = context.fields;
  return saturate(fields.sedimentaryBasin[cell] * 0.54 + fields.crustAge[cell] / 4200 * 0.22 + fields.volcanism[cell] * 0.16 + fields.uplift[cell] * 0.12);
}
