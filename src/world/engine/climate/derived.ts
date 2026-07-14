import type { GridGeometry } from "../../../lib/field/grid";
import { bell, saturate } from "../../../lib/math/scalar";
import type { DistancePropagation } from "../shared/distance";

export interface ClimateDerivedFields {
  aridity: Float32Array;
  windSpeed: Float32Array;
  storminess: Float32Array;
  freezeThaw: Float32Array;
}

export function createClimateDerivedFields(
  grid: GridGeometry,
  temperature: Float32Array,
  seasonality: Float32Array,
  precipitation: Float32Array,
  seaSurfaceTemperature: Float32Array,
  windX: Float32Array,
  windY: Float32Array,
  coast: DistancePropagation
): ClimateDerivedFields {
  const aridity = new Float32Array(grid.cellCount);
  const windSpeed = new Float32Array(grid.cellCount);
  const storminess = new Float32Array(grid.cellCount);
  const freezeThaw = new Float32Array(grid.cellCount);
  for (let cell = 0; cell < grid.cellCount; cell += 1) {
    const latitudeDegrees = Math.abs(grid.latitudeRadians[Math.floor(cell / grid.width)] * 180 / Math.PI);
    const potentialEvaporation = Math.max(120, 520 + Math.max(temperature[cell], -5) * 38 + seasonality[cell] * 9);
    aridity[cell] = saturate(1 - precipitation[cell] / potentialEvaporation);
    windSpeed[cell] = saturate(Math.hypot(windX[cell], windY[cell]));
    const frontal = bell(latitudeDegrees, 48, 13) * windSpeed[cell];
    const nearestOcean = coast.nearestSource[cell];
    const nearbySst = nearestOcean >= 0 ? seaSurfaceTemperature[nearestOcean] : -3;
    const tropical = bell(latitudeDegrees, 14, 7) * saturate((nearbySst - 26.5) / 3) * Math.exp(-coast.distanceKm[cell] / 550);
    storminess[cell] = saturate(frontal * 0.72 + tropical * 0.9);
    const winterCold = temperature[cell] - seasonality[cell] * 0.5;
    const summerWarm = temperature[cell] + seasonality[cell] * 0.5;
    freezeThaw[cell] = saturate(bell(temperature[cell], 0, 9) * 0.65 + Number(winterCold < 0 && summerWarm > 0) * 0.35);
  }
  return { aridity, windSpeed, storminess, freezeThaw };
}
