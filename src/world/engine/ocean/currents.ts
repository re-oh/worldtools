import type { GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";
import { cardinalNeighbor, diffuseField } from "../shared/fieldOps";
import type { VectorField2 } from "../climate/circulation";

export interface OceanCirculation extends VectorField2 {
  speed: Float32Array;
  upwelling: Float32Array;
}

export function createOceanCirculation(grid: GridGeometry, waterDepth: Float32Array, winds: VectorField2): OceanCirculation {
  let x = new Float32Array(grid.cellCount);
  let y = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] <= 0) continue;
    const row = Math.floor(cell / grid.width);
    const coriolis = Math.sin(grid.latitudeRadians[row]);
    x[cell] = winds.x[cell] * 0.72 - winds.y[cell] * coriolis * 0.42;
    y[cell] = winds.y[cell] * 0.72 + winds.x[cell] * coriolis * 0.42;
  }

  for (let pass = 0; pass < 8; pass += 1) {
    const nextX = x.slice();
    const nextY = y.slice();
    for (let cell = 0; cell < grid.cellCount; cell += 1) {
      if (waterDepth[cell] <= 0) continue;
      const east = cardinalNeighbor(cell, 1, 0, grid);
      const west = cardinalNeighbor(cell, -1, 0, grid);
      const north = cardinalNeighbor(cell, 0, 1, grid);
      const south = cardinalNeighbor(cell, 0, -1, grid);
      const blockedX = (waterDepth[east] <= 0 ? Math.max(0, x[cell]) : 0) - (waterDepth[west] <= 0 ? Math.max(0, -x[cell]) : 0);
      const blockedY = (waterDepth[north] <= 0 ? Math.max(0, y[cell]) : 0) - (waterDepth[south] <= 0 ? Math.max(0, -y[cell]) : 0);
      const hemisphere = Math.sign(grid.latitudeRadians[Math.floor(cell / grid.width)]) || 1;
      const turn = 0.28 * hemisphere;
      nextX[cell] = x[cell] * 0.74 + (y[cell] * turn - blockedX) * 0.26;
      nextY[cell] = y[cell] * 0.74 + (-x[cell] * turn - blockedY) * 0.26;
    }
    x = nextX;
    y = nextY;
  }

  const speed = new Float32Array(grid.cellCount);
  const rawUpwelling = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    if (waterDepth[cell] <= 0) continue;
    const east = cardinalNeighbor(cell, 1, 0, grid);
    const west = cardinalNeighbor(cell, -1, 0, grid);
    const north = cardinalNeighbor(cell, 0, 1, grid);
    const south = cardinalNeighbor(cell, 0, -1, grid);
    const divergence = (x[east] - x[west] + y[north] - y[south]) * 0.5;
    const latitude = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)]);
    const equatorial = Math.exp(-Math.pow(latitude / 0.12, 2));
    const coastal = Number(waterDepth[east] <= 0 || waterDepth[west] <= 0 || waterDepth[north] <= 0 || waterDepth[south] <= 0);
    speed[cell] = saturate(Math.hypot(x[cell], y[cell]) * 0.85);
    rawUpwelling[cell] = saturate(Math.max(0, divergence) * 1.8 + equatorial * 0.45 + coastal * Math.abs(y[cell]) * 0.28);
  }
  return { x, y, speed, upwelling: diffuseField(rawUpwelling, grid, 2, 0.62) };
}
