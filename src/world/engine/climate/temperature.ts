import type { GridGeometry } from "../../../lib/field/grid";
import { clamp } from "../../../lib/math/scalar";
import type { ConstraintSet } from "../../constraints";
import type { WorldRecipe } from "../../recipe";
import type { DistancePropagation } from "../shared/distance";

export interface TemperatureFields {
  mean: Float32Array;
  seasonality: Float32Array;
}

export function createTemperatureFields(
  recipe: WorldRecipe,
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  seaSurfaceTemperature: Float32Array,
  coast: DistancePropagation,
  constraints: ConstraintSet
): TemperatureFields {
  const mean = new Float32Array(grid.cellCount);
  const seasonality = new Float32Array(grid.cellCount);
  const solarScale = Math.pow(recipe.solarConstantWm2 / 1361, 0.25);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const latitude = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)]);
    const radiative = 29.5 * solarScale - Math.pow(Math.sin(latitude), 1.18) * 43 + recipe.greenhouseOffsetC;
    if (waterDepth[cell] > 0) {
      mean[cell] = clamp(seaSurfaceTemperature[cell] + recipe.greenhouseOffsetC * 0.35, -3, 38);
      seasonality[cell] = 2 + Math.sin(latitude) * recipe.axialTiltDegrees * 0.35;
      continue;
    }
    const nearest = coast.nearestSource[cell];
    const marineInfluence = Math.exp(-coast.distanceKm[cell] / 850);
    const marineTemperature = nearest >= 0 ? seaSurfaceTemperature[nearest] : radiative;
    const lapse = Math.max(0, elevation[cell]) * 0.0064;
    mean[cell] = clamp(radiative * (1 - marineInfluence * 0.48) + marineTemperature * marineInfluence * 0.48 - lapse + constraints.temperature[cell], -58, 48);
    seasonality[cell] = clamp(3 + Math.sin(latitude) * recipe.axialTiltDegrees * 0.92 + (1 - marineInfluence) * 15, 1, 58);
  }
  return { mean, seasonality };
}
