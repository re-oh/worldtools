import { calculateSlope, updateWaterDepth } from "../shared/fieldOps";
import type { WorldEngineStage } from "../stage";
import { createPersistentIce } from "./ice";
import { applyGlacialLandforms } from "./landforms";

export const cryosphereStage: WorldEngineStage = {
  id: "cryosphere",
  label: "Ice and glacial landforms",
  detail: "Accumulating ice, routing glacier flow, carving valleys, and depositing till",
  run(context, report) {
    const fields = context.fields;
    fields.ice = createPersistentIce(
      context.recipe,
      context.grid,
      fields.elevation,
      fields.waterDepth,
      fields.temperature,
      fields.precipitation,
      context.constraints
    );
    report(0.58, "Persistent ice and downslope glacier flow equilibrated");
    const landforms = applyGlacialLandforms(
      context.grid,
      fields.elevation,
      fields.waterDepth,
      fields.slope,
      fields.ice,
      context.recipe.erosionStrength * context.recipe.glaciation
    );
    fields.glacialErosion = landforms.erosion;
    fields.glacialDeposit = landforms.deposit;
    updateWaterDepth(fields.elevation, fields.waterDepth);
    fields.slope = calculateSlope(fields.elevation, context.grid);
    report(1, "Glacial valleys, cirques, moraines, till, and outwash proxies applied");
  }
};
