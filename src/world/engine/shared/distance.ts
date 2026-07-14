import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";

export interface DistancePropagation {
  distanceKm: Float32Array;
  nearestSource: Int32Array;
}

export function propagateDistance(grid: GridGeometry, isSource: (cell: number) => boolean): DistancePropagation {
  const steps = new Int32Array(grid.cellCount).fill(-1);
  const nearestSource = new Int32Array(grid.cellCount).fill(-1);
  const queue = new Int32Array(grid.cellCount);
  let head = 0;
  let tail = 0;
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (!isSource(cell)) continue;
    steps[cell] = 0;
    nearestSource[cell] = cell;
    queue[tail++] = cell;
  }
  const neighbors = new Int32Array(8);
  while (head < tail) {
    const cell = queue[head++];
    const count = writeNeighbors(cell, grid, neighbors);
    for (let offset = 0; offset < count; offset += 1) {
      const next = neighbors[offset];
      if (steps[next] >= 0) continue;
      steps[next] = steps[cell] + 1;
      nearestSource[next] = nearestSource[cell];
      queue[tail++] = next;
    }
  }
  const distanceKm = new Float32Array(grid.cellCount);
  const nominalCellKm = Math.sqrt(grid.cellAreaKm2);
  for (let cell = 0; cell < grid.cellCount; cell += 1) distanceKm[cell] = Math.max(0, steps[cell]) * nominalCellKm;
  return { distanceKm, nearestSource };
}
