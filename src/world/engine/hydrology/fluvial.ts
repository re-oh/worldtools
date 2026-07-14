import type { GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";
import type { DrainageResult } from "./routing";

export interface FluvialChange {
  erosion: Float32Array;
  deposition: Float32Array;
}

export function applyFluvialEpoch(
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  slope: Float32Array,
  routing: DrainageResult,
  erodibility: Float32Array,
  erosionStrength: number,
  transportStrength: number
): FluvialChange {
  const erosion = new Float32Array(grid.cellCount);
  const deposition = new Float32Array(grid.cellCount);
  const load = new Float64Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] > 0) continue;
    const streamPower = Math.pow(routing.normalizedFlow[cell], 0.68) * Math.pow(saturate(slope[cell] * 75), 0.72);
    erosion[cell] = Math.min(95, streamPower * (0.25 + erodibility[cell] * 0.75) * 52 * erosionStrength);
    elevation[cell] -= erosion[cell];
    load[cell] = erosion[cell];
  }
  for (const cell of routing.descending) {
    if (waterDepth[cell] > 0) continue;
    const lowGradient = 1 - saturate(slope[cell] * 110);
    const settling = lowGradient * Math.pow(routing.normalizedFlow[cell], 0.42) * 0.24 / Math.max(0.25, transportStrength);
    const deposited = Math.min(85, load[cell] * settling);
    deposition[cell] += deposited;
    elevation[cell] += deposited;
    const receiver = routing.downstream[cell];
    if (receiver >= 0) load[receiver] += Math.max(0, load[cell] - deposited);
  }
  return { erosion, deposition };
}

export function createRunoff(
  precipitation: Float32Array,
  aridity: Float32Array,
  ice: Float32Array,
  slope: Float32Array,
  waterDepth: Float32Array
): Float32Array {
  const runoff = new Float32Array(precipitation.length);
  for (let cell = 0; cell < runoff.length; cell += 1) {
    if (waterDepth[cell] > 0) continue;
    const infiltrationLoss = 0.68 - saturate(slope[cell] * 80) * 0.24;
    const meltwater = ice[cell] * Math.max(0, precipitation[cell]) * 0.18;
    runoff[cell] = Math.max(1, precipitation[cell] * (1 - aridity[cell] * 0.72) * (1 - infiltrationLoss) + meltwater);
  }
  return runoff;
}
