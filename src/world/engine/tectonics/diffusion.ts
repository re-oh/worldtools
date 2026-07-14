import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";

export function diffuseMaximum(source: Float32Array, grid: GridGeometry, rings: number, decay: number): Float32Array {
  let current = source.slice();
  const neighbors = new Int32Array(8);
  for (let ring = 0; ring < rings; ring += 1) {
    const next = current.slice();
    for (let index = 0; index < grid.cellCount; index += 1) {
      const count = writeNeighbors(index, grid, neighbors);
      let maximum = current[index];
      for (let offset = 0; offset < count; offset += 1) maximum = Math.max(maximum, current[neighbors[offset]] * decay);
      next[index] = maximum;
    }
    current = next;
  }
  return current;
}
