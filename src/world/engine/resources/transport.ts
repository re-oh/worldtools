import { saturate } from "../../../lib/math/scalar";
import type { WorldEngineContext } from "../context";
import type { ResourceSystemModel } from "./types";

export function transportDenseMinerals(context: WorldEngineContext, source: Float32Array): Float32Array {
  const carry = new Float64Array(source.length);
  for (let cell = 0; cell < source.length; cell += 1) carry[cell] = source[cell];
  const order = Array.from({ length: source.length }, (_, cell) => cell)
    .sort((a, b) => context.scratch.filledElevation[b] - context.scratch.filledElevation[a] || b - a);
  const trapped = new Float32Array(source.length);
  let maximum = 0;
  for (const cell of order) {
    if (context.fields.waterDepth[cell] > 0) continue;
    const hydraulicTrap = Math.pow(context.fields.flow[cell], 0.45) * (1 - saturate(context.fields.slope[cell] * 110)) * (0.3 + context.fields.floodplain[cell] * 0.7);
    trapped[cell] = carry[cell] * hydraulicTrap;
    maximum = Math.max(maximum, trapped[cell]);
    const receiver = context.downstream[cell];
    if (receiver >= 0) carry[receiver] += carry[cell] * 0.9;
  }
  if (maximum > 0) for (let cell = 0; cell < trapped.length; cell += 1) trapped[cell] = saturate(Math.log1p(trapped[cell]) / Math.log1p(maximum));
  return trapped;
}

export function createPlacerModels(goldTransport: Float32Array, gemTransport: Float32Array): readonly ResourceSystemModel[] {
  return [
    {
      typeId: "alluvial-placer-au", name: "Alluvial gold placer", resourceClass: "metal", abundance: "mineral", outputs: ["placer", "gold"], geometry: "alluvial-trap",
      host: "Basal gravel, point bar, bedrock riffle, or abandoned channel", setting: "High-energy stream reach immediately below a hydraulic gradient break", formation: "Gold liberated from upstream veins survived transport and became mechanically concentrated where velocity fell, especially at inside bends, below rapids, and on irregular bedrock.", commodities: ["gold", "platinum-group elements"],
      minimumScore: 0.52, maximumOccurrences: 10, minimumSpacingKm: 230, radiusKm: [1, 28], ageMa: [0.0001, 12], depthMeters: [0, 35], thicknessMeters: [0.3, 12],
      score: (c, i) => goldTransport[i] * Math.pow(c.fields.flow[i], 0.4) * (0.45 + c.fields.floodplain[i] * 0.55)
    },
    {
      typeId: "placer-gem", name: "Alluvial gemstone placer", resourceClass: "gemstone", abundance: "mineral", outputs: ["placer", "gemstone"], geometry: "alluvial-trap",
      host: "Durable heavy-mineral gravel in river or beach sediment", setting: "Drainage downstream from gem-bearing metamorphic or pegmatite terrain", formation: "Weathering released resistant gem minerals that survived abrasion and were density-sorted into channel lags, point bars, terraces, or shoreline placers.", commodities: ["sapphire", "ruby", "garnet", "zircon", "tourmaline"],
      minimumScore: 0.5, maximumOccurrences: 8, minimumSpacingKm: 250, radiusKm: [1, 35], ageMa: [0.0001, 18], depthMeters: [0, 45], thicknessMeters: [0.4, 14],
      score: (c, i) => gemTransport[i] * Math.pow(c.fields.flow[i], 0.35) * (0.4 + c.fields.sediment[i] / 180 + c.fields.floodplain[i] * 0.35)
    }
  ];
}
