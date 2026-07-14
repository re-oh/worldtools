import { clamp, saturate } from "../../../lib/math/scalar";
import { fractalNoise3 } from "../../../lib/random/deterministic";
import type { GridGeometry } from "../../../lib/field/grid";
import type { WorldRecipe } from "../../recipe";

export interface ProvinceFields {
  sedimentaryBasin: Float32Array;
  carbonatePlatform: Float32Array;
  lithology: Float32Array;
}

export function classifyProvinces(
  recipe: WorldRecipe,
  grid: GridGeometry,
  continentalness: Float32Array,
  crustAge: Float32Array,
  convergence: Float32Array,
  divergence: Float32Array,
  shear: Float32Array,
  uplift: Float32Array,
  volcanism: Float32Array
): ProvinceFields {
  const sedimentaryBasin = new Float32Array(grid.cellCount);
  const carbonatePlatform = new Float32Array(grid.cellCount);
  const lithology = new Float32Array(grid.cellCount);
  for (let index = 0; index < grid.cellCount; index += 1) {
    const continent = saturate(continentalness[index] * 2.2);
    const margin = 1 - Math.min(1, Math.abs(continentalness[index]) * 3);
    const stable = saturate(1 - Math.max(convergence[index], divergence[index], shear[index], volcanism[index]));
    const provinceNoise = fractalNoise3(grid.x[index], grid.y[index], grid.z[index], recipe.seed + 347, 3.2, 2);
    const riftBasin = divergence[index] * (0.35 + continent * 0.65);
    const pullApart = shear[index] * divergence[index];
    const foreland = uplift[index] * (1 - convergence[index]) * continent * 0.64;
    const basin = saturate(Math.max(riftBasin, pullApart, foreland) + Math.max(0, provinceNoise) * 0.12);
    const platform = saturate((continent * 0.5 + margin * 0.5) * stable * (0.72 + provinceNoise * 0.22) * (1 - uplift[index]));
    sedimentaryBasin[index] = basin;
    carbonatePlatform[index] = platform;

    if (margin > 0.5 && convergence[index] > 0.45 && crustAge[index] > 150) lithology[index] = 5;
    else if (basin > 0.48) lithology[index] = 4;
    else if (uplift[index] > 0.5 && convergence[index] > 0.35) lithology[index] = 3;
    else if (volcanism[index] > 0.48 && continent > 0.28) lithology[index] = 2;
    else if (volcanism[index] > 0.32 || continentalness[index] < 0) lithology[index] = 1;
    else lithology[index] = clamp(Math.round(2 + provinceNoise), 1, 3);
  }
  return { sedimentaryBasin, carbonatePlatform, lithology };
}
