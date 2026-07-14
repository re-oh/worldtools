import type { GridGeometry } from "../../../lib/field/grid";
import { clamp, saturate } from "../../../lib/math/scalar";
import type { ConstraintSet } from "../../constraints";
import type { WorldRecipe } from "../../recipe";
import { cardinalNeighbor } from "../shared/fieldOps";

export function createPrecipitation(
  recipe: WorldRecipe,
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  temperature: Float32Array,
  windX: Float32Array,
  windY: Float32Array,
  constraints: ConstraintSet
): Float32Array {
  let humidity = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) humidity[cell] = waterDepth[cell] > 0 ? saturate((temperature[cell] + 4) / 30) : 0.08;
  const condensation = new Float32Array(grid.cellCount);

  const passes = Math.max(18, Math.round(grid.width / 20));
  for (let pass = 0; pass < passes; pass += 1) {
    const next = humidity.slice();
    for (let cell = 0; cell < grid.cellCount; cell += 1) {
      if (waterDepth[cell] > 0) {
        next[cell] = Math.max(humidity[cell] * 0.85, saturate((temperature[cell] + 4) / 30) * 0.92);
        continue;
      }
      const upstreamX = cardinalNeighbor(cell, windX[cell] >= 0 ? -1 : 1, 0, grid);
      const upstreamY = cardinalNeighbor(cell, 0, windY[cell] >= 0 ? -1 : 1, grid);
      const xWeight = Math.abs(windX[cell]);
      const yWeight = Math.abs(windY[cell]);
      const incoming = (humidity[upstreamX] * xWeight + humidity[upstreamY] * yWeight + humidity[cell] * 0.25) / Math.max(0.25, xWeight + yWeight + 0.25);
      const upstreamElevation = (elevation[upstreamX] * xWeight + elevation[upstreamY] * yWeight) / Math.max(0.01, xWeight + yWeight);
      const orographicLift = saturate((elevation[cell] - upstreamElevation) / 1800) * recipe.orographicStrength;
      const warmConvection = saturate((temperature[cell] - 8) / 24) * 0.035;
      const loss = clamp(0.018 + orographicLift * 0.16 + warmConvection, 0.01, 0.48);
      condensation[cell] = condensation[cell] * 0.72 + incoming * loss * 0.28;
      next[cell] = clamp(incoming * (1 - loss) + 0.006 * recipe.moistureTransport, 0.01, 1);
    }
    humidity = next;
  }

  const precipitation = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const latitude = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)]);
    const subtropicalDrying = Math.exp(-Math.pow((latitude * 180 / Math.PI - 27) / 10, 2));
    const tropicalConvection = Math.exp(-Math.pow(latitude / 0.28, 2)) * saturate((temperature[cell] + 4) / 30);
    const amount = 55 + humidity[cell] * (430 + tropicalConvection * 1900) + condensation[cell] * 6800;
    precipitation[cell] = clamp(amount * (1 - subtropicalDrying * 0.56) * recipe.moistureTransport + constraints.moisture[cell] * 900, 20, 5200);
  }
  return precipitation;
}
