import type { WorldEngineStage } from "../stage";
import { createSoilProfiles } from "./profiles";

export const soilsStage: WorldEngineStage = {
  id: "soils",
  label: "Soils and regolith",
  detail: "Forming soil from parent rock, climate, leaching, ash, alluvium, and glacial material",
  run(context, report) {
    const profiles = createSoilProfiles(context.recipe, context.fields);
    context.fields.weathering = profiles.weathering;
    context.fields.regolithDepth = profiles.regolithDepth;
    context.fields.regolithMoisture = profiles.regolithMoisture;
    context.fields.clayFraction = profiles.clayFraction;
    context.fields.organicCarbon = profiles.organicCarbon;
    context.fields.soilPH = profiles.soilPH;
    context.fields.soilType = profiles.soilType;
    context.fields.mineralFertility = profiles.mineralFertility;
    context.fields.texture = profiles.texture;
    report(1, "Soil orders, texture, pH, moisture, carbon, depth, and fertility resolved");
  }
};
