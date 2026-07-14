import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";
import type { ConstraintSet } from "../../constraints";
import type { WorldRecipe } from "../../recipe";

export function createPersistentIce(
  recipe: WorldRecipe,
  grid: GridGeometry,
  elevation: Float32Array,
  waterDepth: Float32Array,
  temperature: Float32Array,
  precipitation: Float32Array,
  constraints: ConstraintSet
): Float32Array {
  let ice = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const cold = saturate((-temperature[cell] - 1) / 18);
    const snow = saturate(precipitation[cell] / 1100) * cold;
    const latitude = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)]);
    const polarSeaIce = waterDepth[cell] > 0 ? saturate((latitude - 1.05) * 3.2) * saturate((-temperature[cell] + 1) / 9) : 0;
    const landIce = waterDepth[cell] === 0 ? saturate((snow * 0.78 + cold * 0.38 + Math.max(0, elevation[cell]) / 9000) * recipe.glaciation + constraints.ice[cell]) : 0;
    ice[cell] = Math.max(polarSeaIce, landIce);
  }

  const neighbors = new Int32Array(8);
  for (let pass = 0; pass < 9; pass += 1) {
    const next = new Float32Array(ice.length);
    for (let cell = 0; cell < grid.cellCount; cell += 1) {
      if (waterDepth[cell] > 0) {
        next[cell] = ice[cell];
        continue;
      }
      const count = writeNeighbors(cell, grid, neighbors);
      let receiver = -1;
      let lowestSurface = elevation[cell] + ice[cell] * 900;
      for (let offset = 0; offset < count; offset += 1) {
        const neighbor = neighbors[offset];
        if (waterDepth[neighbor] > 0) continue;
        const surface = elevation[neighbor] + ice[neighbor] * 900;
        if (surface < lowestSurface) {
          lowestSurface = surface;
          receiver = neighbor;
        }
      }
      const mobile = ice[cell] * saturate((ice[cell] - 0.12) * 2.2) * 0.14;
      next[cell] += ice[cell] - mobile;
      if (receiver >= 0) next[receiver] += mobile;
      else next[cell] += mobile;
    }
    for (let cell = 0; cell < grid.cellCount; cell += 1) ice[cell] = saturate(next[cell]);
  }
  return ice;
}
