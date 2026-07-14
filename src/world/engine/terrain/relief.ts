import type { GridGeometry } from "../../../lib/field/grid";
import { clamp, saturate } from "../../../lib/math/scalar";
import { fractalNoise3, ridgedNoise3 } from "../../../lib/random/deterministic";
import type { ConstraintSet } from "../../constraints";
import type { FieldMap } from "../../model";
import type { WorldRecipe } from "../../recipe";

export function createRelief(recipe: WorldRecipe, grid: GridGeometry, fields: FieldMap, constraints: ConstraintSet): Float32Array {
  const elevation = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const continental = fields.continentalness[cell] >= 0;
    const affinity = Math.abs(fields.continentalness[cell]);
    const broadNoise = fractalNoise3(grid.x[cell], grid.y[cell], grid.z[cell], recipe.seed + 401, 2.7, 4, 0.5);
    const ruggedNoise = ridgedNoise3(grid.x[cell], grid.y[cell], grid.z[cell], recipe.seed + 409, 8.5, 3);
    let base: number;
    if (continental) {
      const isostasy = Math.max(-180, (fields.crustThickness[cell] - 28) * 72);
      base = 260 + affinity * 860 + isostasy + broadNoise * 480;
    } else {
      const thermalSubsidence = 350 * Math.sqrt(Math.max(0, fields.crustAge[cell]));
      base = -(2450 + thermalSubsidence) + broadNoise * 260;
    }
    const orogen = Math.pow(fields.uplift[cell], 1.35) * (3300 + affinity * 1900) * recipe.tectonicActivity;
    const volcanicRelief = Math.pow(fields.volcanism[cell], 1.7) * (continental ? 1300 : 2600);
    const trench = fields.convergence[cell] * saturate(-fields.continentalness[cell] * 2.5) * 2800;
    const rift = fields.divergence[cell] * saturate(fields.continentalness[cell] * 2) * 850;
    const subsidence = fields.sedimentaryBasin[cell] * (continental ? 760 : 380);
    const smallScale = (ruggedNoise - 0.5) * (continental ? 360 : 190) * (0.25 + fields.uplift[cell]);
    elevation[cell] = clamp(base + orogen + volcanicRelief - trench - rift - subsidence + smallScale + constraints.elevation[cell] - recipe.seaLevelMeters, -9200, 7800);
  }
  return elevation;
}
