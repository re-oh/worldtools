import { assertFieldLength } from "../lib/field/field";
import type { GridSpec } from "../lib/field/grid";
import type { WorldStageId } from "./model";

export const CONSTRAINT_IDS = [
  "continentalness",
  "elevation",
  "tectonicActivity",
  "oceanTemperature",
  "temperature",
  "moisture",
  "ice",
  "vegetation",
  "resourceFavorability"
] as const;
export type ConstraintId = (typeof CONSTRAINT_IDS)[number];
export type ConstraintSet = Record<ConstraintId, Float32Array>;

export interface ConstraintDefinition {
  id: ConstraintId;
  label: string;
  units: string;
  minimum: number;
  maximum: number;
  defaultBrushStrength: number;
  invalidates: WorldStageId;
}

export const CONSTRAINT_DEFINITIONS: Record<ConstraintId, ConstraintDefinition> = {
  continentalness: {
    id: "continentalness",
    label: "Continental affinity",
    units: "bias",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.18,
    invalidates: "geology"
  },
  elevation: {
    id: "elevation",
    label: "Elevation offset",
    units: "m",
    minimum: -4500,
    maximum: 4500,
    defaultBrushStrength: 280,
    invalidates: "terrain"
  },
  tectonicActivity: {
    id: "tectonicActivity",
    label: "Tectonic activity",
    units: "bias",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.15,
    invalidates: "tectonics"
  },
  oceanTemperature: {
    id: "oceanTemperature",
    label: "Ocean temperature anomaly",
    units: "C",
    minimum: -12,
    maximum: 12,
    defaultBrushStrength: 0.8,
    invalidates: "ocean"
  },
  temperature: {
    id: "temperature",
    label: "Temperature anomaly",
    units: "C",
    minimum: -20,
    maximum: 20,
    defaultBrushStrength: 1.5,
    invalidates: "climate"
  },
  moisture: {
    id: "moisture",
    label: "Moisture anomaly",
    units: "ratio",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.12,
    invalidates: "climate"
  },
  ice: {
    id: "ice",
    label: "Persistent ice bias",
    units: "bias",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.12,
    invalidates: "cryosphere"
  },
  vegetation: {
    id: "vegetation",
    label: "Vegetation cover bias",
    units: "bias",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.12,
    invalidates: "ecology"
  },
  resourceFavorability: {
    id: "resourceFavorability",
    label: "Resource favorability",
    units: "bias",
    minimum: -1,
    maximum: 1,
    defaultBrushStrength: 0.1,
    invalidates: "resources"
  }
};

export interface SparseConstraintLayer {
  id: ConstraintId;
  entries: Array<[number, number]>;
}

export function createConstraintSet(cellCount: number): ConstraintSet {
  return Object.fromEntries(CONSTRAINT_IDS.map((id) => [id, new Float32Array(cellCount)])) as ConstraintSet;
}

export function cloneConstraintSet(source: ConstraintSet): ConstraintSet {
  return Object.fromEntries(CONSTRAINT_IDS.map((id) => [id, source[id].slice()])) as ConstraintSet;
}

export function validateConstraintSet(constraints: ConstraintSet, grid: GridSpec): void {
  for (const id of CONSTRAINT_IDS) assertFieldLength(constraints[id], grid, `Constraint ${id}`);
}

export function encodeConstraintSet(constraints: ConstraintSet, epsilon = 1e-5): SparseConstraintLayer[] {
  return CONSTRAINT_IDS.map((id) => {
    const entries: Array<[number, number]> = [];
    const values = constraints[id];
    for (let index = 0; index < values.length; index += 1) {
      if (Math.abs(values[index]) > epsilon) entries.push([index, values[index]]);
    }
    return { id, entries };
  });
}

export function decodeConstraintSet(cellCount: number, encoded: SparseConstraintLayer[]): ConstraintSet {
  const constraints = createConstraintSet(cellCount);
  for (const layer of encoded) {
    const target = constraints[layer.id];
    if (!target) continue;
    const definition = CONSTRAINT_DEFINITIONS[layer.id];
    for (const [index, value] of layer.entries) {
      if (index >= 0 && index < cellCount && Number.isFinite(value)) {
        target[index] = Math.max(definition.minimum, Math.min(definition.maximum, value));
      }
    }
  }
  return constraints;
}

export function constraintCount(constraints: ConstraintSet): number {
  let count = 0;
  for (const id of CONSTRAINT_IDS) for (const value of constraints[id]) if (Math.abs(value) > 1e-5) count += 1;
  return count;
}

export function resampleConstraintSet(
  source: ConstraintSet,
  sourceWidth: number,
  sourceHeight: number,
  targetWidth: number,
  targetHeight: number
): ConstraintSet {
  if (sourceWidth === targetWidth && sourceHeight === targetHeight) return cloneConstraintSet(source);
  const target = createConstraintSet(targetWidth * targetHeight);
  for (const id of CONSTRAINT_IDS) {
    for (let row = 0; row < targetHeight; row += 1) {
      const sourceY = ((row + 0.5) / targetHeight) * sourceHeight - 0.5;
      const y0 = Math.max(0, Math.min(sourceHeight - 1, Math.floor(sourceY)));
      const y1 = Math.max(0, Math.min(sourceHeight - 1, y0 + 1));
      const ty = Math.max(0, Math.min(1, sourceY - y0));
      for (let column = 0; column < targetWidth; column += 1) {
        const sourceX = ((column + 0.5) / targetWidth) * sourceWidth - 0.5;
        const x0 = ((Math.floor(sourceX) % sourceWidth) + sourceWidth) % sourceWidth;
        const x1 = (x0 + 1) % sourceWidth;
        const tx = sourceX - Math.floor(sourceX);
        const top = source[id][y0 * sourceWidth + x0] * (1 - tx) + source[id][y0 * sourceWidth + x1] * tx;
        const bottom = source[id][y1 * sourceWidth + x0] * (1 - tx) + source[id][y1 * sourceWidth + x1] * tx;
        target[id][row * targetWidth + column] = top * (1 - ty) + bottom * ty;
      }
    }
  }
  return target;
}
