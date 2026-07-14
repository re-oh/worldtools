import { calculateSlope, updateWaterDepth } from "../shared/fieldOps";
import type { WorldEngineStage } from "../stage";
import { createRelief } from "./relief";

export const terrainStage: WorldEngineStage = {
  id: "terrain",
  label: "Lithosphere and relief",
  detail: "Resolving isostasy, seafloor subsidence, orogens, rifts, trenches, and basins",
  run(context, report) {
    context.fields.elevation = createRelief(context.recipe, context.grid, context.fields, context.constraints);
    report(0.72, "Crustal buoyancy and tectonic landforms resolved");
    updateWaterDepth(context.fields.elevation, context.fields.waterDepth);
    context.fields.slope = calculateSlope(context.fields.elevation, context.grid);
    report(1, "Relief derivatives and ocean bathymetry complete");
  }
};
