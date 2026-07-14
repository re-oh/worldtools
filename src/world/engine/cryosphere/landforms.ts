import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";

export interface GlacialLandforms {
  erosion: Float32Array;
  deposit: Float32Array;
}

export function applyGlacialLandforms(
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  slope: Float32Array,
  ice: Float32Array,
  strength: number
): GlacialLandforms {
  const erosion = new Float32Array(grid.cellCount);
  const deposit = new Float32Array(grid.cellCount);
  const carved = elevation.slice();
  const neighbors = new Int32Array(8);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] > 0 || ice[cell] < 0.08) continue;
    const count = writeNeighbors(cell, grid, neighbors);
    let neighborMean = 0;
    let margin = 0;
    for (let offset = 0; offset < count; offset += 1) {
      neighborMean += elevation[neighbors[offset]];
      margin = Math.max(margin, ice[cell] - ice[neighbors[offset]]);
    }
    neighborMean /= Math.max(1, count);
    const sliding = Math.pow(ice[cell], 1.4) * saturate(slope[cell] * 85 + 0.08);
    const incision = Math.min(420, sliding * 230 * strength);
    const valleyBroadening = (neighborMean - elevation[cell]) * ice[cell] * 0.13;
    erosion[cell] = Math.max(0, incision - Math.max(0, valleyBroadening));
    carved[cell] = elevation[cell] - incision + valleyBroadening;
    if (margin > 0.16) deposit[cell] = Math.min(140, margin * (0.35 + ice[cell]) * 95 * strength);
  }
  for (let cell = 0; cell < grid.cellCount; cell += 1) elevation[cell] = carved[cell] + deposit[cell];
  return { erosion, deposit };
}
