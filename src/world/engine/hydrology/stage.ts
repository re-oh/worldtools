import { calculateSlope, updateWaterDepth } from "../shared/fieldOps";
import type { WorldEngineStage } from "../stage";
import { buildRiverDeltas } from "./deltas";
import { createRiverLandforms } from "./floodplains";
import { applyFluvialEpoch, createRunoff } from "./fluvial";
import { routeDrainage } from "./routing";

export const hydrologyStage: WorldEngineStage = {
  id: "hydrology",
  label: "Rivers and sediment",
  detail: "Rerouting drainage through erosion epochs, floodplains, channel migration, and deltas",
  run(context, report) {
    const fields = context.fields;
    fields.runoff = createRunoff(fields.precipitation, fields.aridity, fields.ice, fields.slope, fields.waterDepth);
    const totalErosion = new Float32Array(context.grid.cellCount);
    const totalSediment = fields.glacialDeposit.slice();
    const routeChanges = new Uint8Array(context.grid.cellCount);
    let previousRoute = new Int32Array(context.grid.cellCount).fill(-2);
    const epochs = Math.max(2, Math.min(5, Math.round(context.recipe.erosionCycles / 6)));
    const epochStrength = context.recipe.erosionStrength * context.recipe.erosionCycles / (epochs * 18);
    let routing = routeDrainage(context.grid, fields.elevation, fields.waterDepth, fields.runoff);
    for (let epoch = 0; epoch < epochs; epoch += 1) {
      for (let cell = 0; cell < context.grid.cellCount; cell += 1) {
        if (previousRoute[cell] >= -1 && previousRoute[cell] !== routing.downstream[cell]) routeChanges[cell] += 1;
      }
      previousRoute = routing.downstream.slice();
      const erodibility = new Float32Array(context.grid.cellCount);
      for (let cell = 0; cell < erodibility.length; cell += 1) erodibility[cell] = 0.3 + fields.sedimentaryBasin[cell] * 0.45 + fields.freezeThaw[cell] * 0.25;
      const change = applyFluvialEpoch(
        context.grid,
        fields.elevation,
        fields.waterDepth,
        fields.slope,
        routing,
        erodibility,
        epochStrength,
        context.recipe.sedimentTransport
      );
      for (let cell = 0; cell < context.grid.cellCount; cell += 1) {
        totalErosion[cell] += change.erosion[cell];
        totalSediment[cell] += change.deposition[cell];
      }
      updateWaterDepth(fields.elevation, fields.waterDepth);
      fields.slope = calculateSlope(fields.elevation, context.grid);
      routing = routeDrainage(context.grid, fields.elevation, fields.waterDepth, fields.runoff);
      report(0.14 + ((epoch + 1) / epochs) * 0.62, `Fluvial epoch ${epoch + 1} of ${epochs} rerouted`);
    }
    fields.erosion = totalErosion;
    fields.sediment = totalSediment;
    fields.flow = routing.normalizedFlow;
    fields.lakeDepth = routing.lakeDepth;
    context.downstream = routing.downstream;
    context.basin = routing.basin;
    context.scratch.filledElevation = routing.filledElevation;
    const riverLandforms = createRiverLandforms(
      fields.flow,
      fields.slope,
      fields.sediment,
      fields.lakeDepth,
      routeChanges,
      epochs,
      context.recipe.riverMobility
    );
    fields.floodplain = riverLandforms.floodplain;
    fields.channelMobility = riverLandforms.mobility;
    fields.delta = buildRiverDeltas(
      context.grid,
      fields.elevation,
      fields.waterDepth,
      context.downstream,
      fields.flow,
      fields.sediment,
      fields.oceanCurrent,
      context.scratch.currentX,
      context.scratch.currentY,
      context.recipe.sedimentTransport
    );
    updateWaterDepth(fields.elevation, fields.waterDepth);
    fields.slope = calculateSlope(fields.elevation, context.grid);
    report(1, "Acyclic drainage, mobile channels, floodplains, and delta fans complete");
  }
};
