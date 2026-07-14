import type { GridSpec } from "./grid";

export interface FieldStats {
  minimum: number;
  maximum: number;
  mean: number;
}

export function createField(grid: GridSpec, initial = 0): Float32Array {
  const values = new Float32Array(grid.cellCount);
  if (initial !== 0) values.fill(initial);
  return values;
}

export function calculateFieldStats(values: Float32Array): FieldStats {
  let minimum = Number.POSITIVE_INFINITY;
  let maximum = Number.NEGATIVE_INFINITY;
  let sum = 0;
  for (const value of values) {
    minimum = Math.min(minimum, value);
    maximum = Math.max(maximum, value);
    sum += value;
  }
  return { minimum, maximum, mean: sum / Math.max(values.length, 1) };
}

export function copyFieldMap<T extends string>(fields: Record<T, Float32Array>): Record<T, Float32Array> {
  return Object.fromEntries(Object.entries<Float32Array>(fields).map(([id, values]) => [id, values.slice()])) as Record<T, Float32Array>;
}

export function assertFieldLength(values: Float32Array, grid: GridSpec, label: string): void {
  if (values.length !== grid.cellCount) throw new Error(`${label} contains ${values.length} cells; expected ${grid.cellCount}.`);
}
