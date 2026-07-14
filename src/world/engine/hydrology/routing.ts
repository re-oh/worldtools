import { MinHeap } from "../../../lib/collections/MinHeap";
import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";
import { normalizeField } from "../shared/fieldOps";

interface FloodNode {
  cell: number;
  height: number;
}

export interface DrainageResult {
  filledElevation: Float32Array;
  downstream: Int32Array;
  basin: Int32Array;
  lakeDepth: Float32Array;
  accumulation: Float64Array;
  normalizedFlow: Float32Array;
  descending: number[];
}

export function routeDrainage(grid: GridGeometry, elevation: Float32Array, waterDepth: Float32Array, runoff: Float32Array): DrainageResult {
  const filledElevation = elevation.slice();
  const downstream = new Int32Array(grid.cellCount).fill(-1);
  const visited = new Uint8Array(grid.cellCount);
  const heap = new MinHeap<FloodNode>((a, b) => a.height - b.height || a.cell - b.cell);
  let outletCount = 0;
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] <= 0) continue;
    visited[cell] = 1;
    heap.push({ cell, height: elevation[cell] });
    outletCount += 1;
  }
  if (outletCount === 0) {
    let lowest = 0;
    for (let cell = 1; cell < grid.cellCount; cell += 1) if (elevation[cell] < elevation[lowest]) lowest = cell;
    visited[lowest] = 1;
    heap.push({ cell: lowest, height: elevation[lowest] });
  }

  const neighbors = new Int32Array(8);
  const epsilon = 0.002;
  while (heap.size > 0) {
    const node = heap.pop()!;
    const count = writeNeighbors(node.cell, grid, neighbors);
    for (let offset = 0; offset < count; offset += 1) {
      const next = neighbors[offset];
      if (visited[next]) continue;
      visited[next] = 1;
      downstream[next] = node.cell;
      filledElevation[next] = Math.max(elevation[next], node.height + epsilon);
      heap.push({ cell: next, height: filledElevation[next] });
    }
  }

  const descending = Array.from({ length: grid.cellCount }, (_, cell) => cell)
    .sort((a, b) => filledElevation[b] - filledElevation[a] || b - a);
  const accumulation = new Float64Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] === 0) accumulation[cell] = Math.max(1, runoff[cell]);
  }
  for (const cell of descending) {
    const receiver = downstream[cell];
    if (receiver >= 0) accumulation[receiver] += accumulation[cell];
  }
  const rawLandFlow = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) if (waterDepth[cell] === 0) rawLandFlow[cell] = accumulation[cell];
  const normalizedFlow = normalizeField(rawLandFlow, true);

  const lakeDepth = new Float32Array(grid.cellCount);
  const basin = new Int32Array(grid.cellCount).fill(-1);
  const ascending = descending.slice().reverse();
  for (const cell of ascending) {
    if (waterDepth[cell] > 0) {
      basin[cell] = cell;
      continue;
    }
    lakeDepth[cell] = Math.max(0, filledElevation[cell] - elevation[cell]);
    const receiver = downstream[cell];
    basin[cell] = receiver >= 0 ? (waterDepth[receiver] > 0 ? receiver : basin[receiver]) : cell;
  }
  return { filledElevation, downstream, basin, lakeDepth, accumulation, normalizedFlow, descending };
}
