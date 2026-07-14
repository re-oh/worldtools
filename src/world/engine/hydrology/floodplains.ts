import { saturate } from "../../../lib/math/scalar";

export interface RiverLandforms {
  floodplain: Float32Array;
  mobility: Float32Array;
}

export function createRiverLandforms(
  flow: Float32Array,
  slope: Float32Array,
  sediment: Float32Array,
  lakeDepth: Float32Array,
  routeChanges: Uint8Array,
  epochCount: number,
  mobilityStrength: number
): RiverLandforms {
  const floodplain = new Float32Array(flow.length);
  const mobility = new Float32Array(flow.length);
  for (let cell = 0; cell < flow.length; cell += 1) {
    const lowGradient = 1 - saturate(slope[cell] * 105);
    const alluvium = saturate(sediment[cell] / 90);
    floodplain[cell] = saturate(Math.pow(flow[cell], 0.55) * lowGradient * (0.45 + alluvium * 0.55) + saturate(lakeDepth[cell] / 12) * 0.25);
    const migrationHistory = routeChanges[cell] / Math.max(1, epochCount - 1);
    mobility[cell] = saturate((floodplain[cell] * Math.pow(flow[cell], 0.35) * (0.45 + alluvium * 0.55) + migrationHistory * 0.55) * mobilityStrength);
  }
  return { floodplain, mobility };
}
