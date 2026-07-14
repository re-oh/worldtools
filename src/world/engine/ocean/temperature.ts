import type { GridGeometry } from "../../../lib/field/grid";
import { clamp } from "../../../lib/math/scalar";
import type { ConstraintSet } from "../../constraints";
import { cardinalNeighbor } from "../shared/fieldOps";
import type { OceanCirculation } from "./currents";

export function createSeaSurfaceTemperature(
  grid: GridGeometry,
  waterDepth: Float32Array,
  circulation: OceanCirculation,
  constraints: ConstraintSet,
  solarScale: number,
  heatTransport: number
): Float32Array {
  let temperature = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] <= 0) {
      temperature[cell] = 0;
      continue;
    }
    const latitude = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)]);
    temperature[cell] = 30.5 * solarScale - Math.pow(Math.sin(latitude), 1.2) * 32 - circulation.upwelling[cell] * 5 + constraints.oceanTemperature[cell];
  }
  for (let pass = 0; pass < 10; pass += 1) {
    const next = temperature.slice();
    for (let cell = 0; cell < grid.cellCount; cell += 1) {
      if (waterDepth[cell] <= 0) continue;
      const dx = circulation.x[cell] >= 0 ? -1 : 1;
      const dy = circulation.y[cell] >= 0 ? -1 : 1;
      const upstreamX = cardinalNeighbor(cell, dx, 0, grid);
      const upstreamY = cardinalNeighbor(cell, 0, dy, grid);
      let advected = temperature[cell];
      let weight = 1;
      if (waterDepth[upstreamX] > 0) { advected += temperature[upstreamX] * Math.abs(circulation.x[cell]); weight += Math.abs(circulation.x[cell]); }
      if (waterDepth[upstreamY] > 0) { advected += temperature[upstreamY] * Math.abs(circulation.y[cell]); weight += Math.abs(circulation.y[cell]); }
      next[cell] = clamp(temperature[cell] * 0.78 + (advected / weight) * 0.22 * heatTransport + temperature[cell] * 0.22 * (1 - heatTransport), -2.5, 35);
    }
    temperature = next;
  }
  return temperature;
}
