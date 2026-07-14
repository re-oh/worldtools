import { clamp } from "../../../lib/math/scalar";
import type { WorldEngineStage } from "../stage";
import { createVegetation } from "./vegetation";

export const ecologyStage: WorldEngineStage = {
  id: "ecology",
  label: "Vegetation and biomes",
  detail: "Resolving plant cover from energy, water, soils, seasonality, disturbance, and drainage",
  run(context, report) {
    const cover = createVegetation(context.recipe, context.fields, context.constraints);
    context.fields.vegetationCover = cover.vegetation;
    context.fields.forestCover = cover.forest;
    context.fields.grassCover = cover.grass;
    context.fields.wetlandCover = cover.wetland;
    context.fields.biome = cover.biome;
    for (let cell = 0; cell < context.grid.cellCount; cell += 1) {
      if (context.fields.waterDepth[cell] > 0) continue;
      const litter = cover.vegetation[cell] * (0.025 + (1 - Math.max(0, context.fields.temperature[cell]) / 45) * 0.035);
      const preserved = cover.wetland[cell] * 0.07;
      context.fields.organicCarbon[cell] = clamp(context.fields.organicCarbon[cell] * 0.55 + litter + preserved, 0.001, 0.24);
    }
    report(1, "Forests, grasslands, plains, wetlands, tundra, desert, and alpine biomes resolved");
  }
};
