import { fractalNoise3 } from "../../../lib/random/deterministic";
import { clamp } from "../../../lib/math/scalar";
import type { GridGeometry } from "../../../lib/field/grid";
import type { ConstraintSet } from "../../constraints";
import type { PlateModel } from "../../model";
import type { WorldRecipe } from "../../recipe";

export interface CrustFields {
  continentalness: Float32Array;
  crustAge: Float32Array;
  crustThickness: Float32Array;
}

export function createCrustFields(
  recipe: WorldRecipe,
  grid: GridGeometry,
  plateIds: Float32Array,
  plates: PlateModel[],
  constraints: ConstraintSet
): CrustFields {
  const raw = new Float32Array(grid.cellCount);
  for (let index = 0; index < grid.cellCount; index += 1) {
    const plate = plates[Math.trunc(plateIds[index])];
    const macro = fractalNoise3(grid.x[index], grid.y[index], grid.z[index], recipe.seed + 211, 1.25, 4, 0.54);
    const shelf = fractalNoise3(grid.x[index], grid.y[index], grid.z[index], recipe.seed + 223, 3.4, 3, 0.5);
    raw[index] = macro * 0.72 + shelf * 0.2 + plate.continentalBias * 0.38 + constraints.continentalness[index];
  }

  const ordered = raw.slice().sort();
  const thresholdIndex = Math.max(0, Math.min(ordered.length - 1, Math.floor((1 - recipe.continentalFraction) * ordered.length)));
  const threshold = ordered[thresholdIndex];
  const continentalness = new Float32Array(grid.cellCount);
  const crustAge = new Float32Array(grid.cellCount);
  const crustThickness = new Float32Array(grid.cellCount);
  const maximumContinentalAge = Math.min(4000, recipe.worldAgeGa * 1000 * 0.94);

  for (let index = 0; index < grid.cellCount; index += 1) {
    const affinity = clamp((raw[index] - threshold) * 2.7, -1, 1);
    const continental = Math.max(0, affinity);
    const transition = 1 - Math.min(1, Math.abs(affinity) * 2.2);
    const ageNoise = fractalNoise3(grid.x[index], grid.y[index], grid.z[index], recipe.seed + 227, 2.1, 2);
    continentalness[index] = affinity;
    crustThickness[index] = clamp(7 + continental * 39 + transition * 9 + ageNoise * (2 + continental * 3), 5, 62);
    crustAge[index] = continental > 0
      ? clamp(420 + continental * 1050 + (ageNoise + 1) * 0.5 * Math.max(0, maximumContinentalAge - 1470), 300, maximumContinentalAge)
      : 35 + (ageNoise + 1) * 72;
  }
  return { continentalness, crustAge, crustThickness };
}
