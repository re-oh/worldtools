import type { WorldEngineStage } from "../stage";
import { createCrustFields } from "./continents";
import { createPlatePartition } from "./plates";

export const geologyStage: WorldEngineStage = {
  id: "geology",
  label: "Crust and plates",
  detail: "Growing connected plates and balancing continental crust",
  run(context, report) {
    const partition = createPlatePartition(context.recipe, context.grid);
    context.plates = partition.plates;
    context.fields.plateId = partition.plateIds;
    report(0.62, "Connected plate partition complete");
    const crust = createCrustFields(context.recipe, context.grid, partition.plateIds, partition.plates, context.constraints);
    context.fields.continentalness = crust.continentalness;
    context.fields.crustAge = crust.crustAge;
    context.fields.crustThickness = crust.crustThickness;
    report(1, "Continental fraction and crustal provinces resolved");
  }
};
