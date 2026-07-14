import type { GridGeometry } from "../../../lib/field/grid";
import { saturate } from "../../../lib/math/scalar";

export interface VectorField2 {
  x: Float32Array;
  y: Float32Array;
}

export function createPlanetaryWinds(grid: GridGeometry, axialTiltDegrees: number): VectorField2 {
  const x = new Float32Array(grid.cellCount);
  const y = new Float32Array(grid.cellCount);
  const tiltScale = 0.8 + axialTiltDegrees / 90;
  for (let row = 0; row < grid.height; row += 1) {
    const latitude = grid.latitudeRadians[row];
    const absoluteDegrees = Math.abs(latitude) * 180 / Math.PI;
    const hemisphere = Math.sign(latitude) || 1;
    let zonal: number;
    let meridional: number;
    if (absoluteDegrees < 30) {
      zonal = -0.86;
      meridional = -hemisphere * 0.28;
    } else if (absoluteDegrees < 60) {
      zonal = 0.9;
      meridional = hemisphere * 0.2;
    } else {
      zonal = -0.66;
      meridional = -hemisphere * 0.18;
    }
    const convergence = 0.72 + saturate(Math.abs(absoluteDegrees - 30) / 35) * 0.18;
    for (let column = 0; column < grid.width; column += 1) {
      const cell = row * grid.width + column;
      x[cell] = zonal * convergence;
      y[cell] = meridional * tiltScale;
    }
  }
  return { x, y };
}
