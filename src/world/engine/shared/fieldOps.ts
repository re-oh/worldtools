import { writeNeighbors, type GridGeometry, type GridSpec } from "../../../lib/field/grid";
import { clamp, inverseLerp, saturate } from "../../../lib/math/scalar";

export function calculateSlope(elevation: Float32Array, grid: GridGeometry): Float32Array {
  const slope = new Float32Array(grid.cellCount);
  const neighbors = new Int32Array(8);
  const cellMeters = Math.sqrt(grid.cellAreaKm2) * 1000;
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const count = writeNeighbors(cell, grid, neighbors);
    let greatest = 0;
    for (let offset = 0; offset < count; offset += 1) {
      const neighbor = neighbors[offset];
      greatest = Math.max(greatest, Math.abs(elevation[cell] - elevation[neighbor]) / cellMeters);
    }
    slope[cell] = greatest;
  }
  return slope;
}

export function diffuseField(source: Float32Array, grid: GridSpec, passes: number, centerWeight = 0.5): Float32Array {
  let current = source.slice();
  let next = new Float32Array(source.length);
  const neighbors = new Int32Array(8);
  for (let pass = 0; pass < passes; pass += 1) {
    for (let cell = 0; cell < grid.cellCount; cell += 1) {
      const count = writeNeighbors(cell, grid, neighbors);
      let sum = 0;
      for (let offset = 0; offset < count; offset += 1) sum += current[neighbors[offset]];
      next[cell] = current[cell] * centerWeight + (sum / Math.max(1, count)) * (1 - centerWeight);
    }
    [current, next] = [next, current];
  }
  return current;
}

export function normalizeField(source: Float32Array, logarithmic = false): Float32Array {
  let minimum = Number.POSITIVE_INFINITY;
  let maximum = Number.NEGATIVE_INFINITY;
  const transformed = new Float32Array(source.length);
  for (let index = 0; index < source.length; index += 1) {
    const value = logarithmic ? Math.log1p(Math.max(0, source[index])) : source[index];
    transformed[index] = value;
    minimum = Math.min(minimum, value);
    maximum = Math.max(maximum, value);
  }
  for (let index = 0; index < source.length; index += 1) transformed[index] = saturate(inverseLerp(minimum, maximum, transformed[index]));
  return transformed;
}

export function maxInto(target: Float32Array, source: Float32Array): void {
  for (let index = 0; index < target.length; index += 1) target[index] = Math.max(target[index], source[index]);
}

export function updateWaterDepth(elevation: Float32Array, waterDepth: Float32Array): void {
  for (let index = 0; index < elevation.length; index += 1) waterDepth[index] = Math.max(0, -elevation[index]);
}

export function cardinalNeighbor(cell: number, dx: number, dy: number, grid: GridSpec): number {
  const row = Math.floor(cell / grid.width);
  const column = cell % grid.width;
  const nextRow = clamp(row + dy, 0, grid.height - 1);
  const nextColumn = ((column + dx) % grid.width + grid.width) % grid.width;
  return nextRow * grid.width + nextColumn;
}
