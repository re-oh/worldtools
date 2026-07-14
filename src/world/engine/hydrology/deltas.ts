import type { GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";
import { cardinalNeighbor } from "../shared/fieldOps";

export function buildRiverDeltas(
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  downstream: Int32Array,
  flow: Float32Array,
  sediment: Float32Array,
  oceanCurrent: Float32Array,
  currentX: Float32Array,
  currentY: Float32Array,
  transportStrength: number
): Float32Array {
  const delta = new Float32Array(grid.cellCount);
  const cellKm = Math.sqrt(grid.cellAreaKm2);
  for (let mouth = 0; mouth < grid.cellCount; mouth += 1) {
    const firstOcean = downstream[mouth];
    if (waterDepth[mouth] > 0 || firstOcean < 0 || waterDepth[firstOcean] <= 0 || flow[mouth] < 0.54) continue;
    const mouthRow = Math.floor(mouth / grid.width);
    const mouthColumn = mouth % grid.width;
    const oceanRow = Math.floor(firstOcean / grid.width);
    const oceanColumn = firstOcean % grid.width;
    let dx = oceanColumn - mouthColumn;
    if (Math.abs(dx) > grid.width / 2) dx -= Math.sign(dx) * grid.width;
    const dy = oceanRow - mouthRow;
    const length = Math.max(1, Math.hypot(dx, dy));
    const forwardX = dx / length;
    const forwardY = dy / length;
    const reworking = saturate(oceanCurrent[firstOcean] * 0.75);
    const supply = saturate(flow[mouth] * 0.68 + sediment[mouth] / 120);
    const range = Math.max(1, Math.min(12, Math.round((45 + supply * 170) / cellKm)));
    const alongshore = (currentX[firstOcean] * -forwardY + currentY[firstOcean] * forwardX) * reworking;
    for (let step = 1; step <= range; step += 1) {
      const halfWidth = Math.max(0, Math.floor(step * (0.22 + supply * 0.42) * (1 - reworking * 0.55)));
      for (let lateral = -halfWidth; lateral <= halfWidth; lateral += 1) {
        const shifted = lateral + alongshore * step;
        const columnOffset = Math.round(forwardX * step - forwardY * shifted);
        const rowOffset = Math.round(forwardY * step + forwardX * shifted);
        const cell = cardinalNeighbor(mouth, columnOffset, rowOffset, grid);
        if (waterDepth[cell] <= 0) continue;
        const fan = supply * (1 - step / (range + 1)) * (1 - Math.abs(lateral) / Math.max(1, halfWidth + 1));
        delta[cell] = Math.max(delta[cell], fan);
      }
    }
  }
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (delta[cell] <= 0) continue;
    const aggradation = Math.min(Math.max(0, waterDepth[cell] - 0.5), delta[cell] * 115 * transportStrength);
    elevation[cell] += aggradation;
    sediment[cell] += aggradation;
  }
  return delta;
}
