import { createPlanetaryWinds } from "../climate/circulation";
import type { WorldEngineStage } from "../stage";
import { createOceanCirculation } from "./currents";
import { createSeaSurfaceTemperature } from "./temperature";

export const oceanStage: WorldEngineStage = {
  id: "ocean",
  label: "Ocean circulation",
  detail: "Resolving wind-driven gyres, coastal deflection, upwelling, and heat transport",
  run(context, report) {
    const winds = createPlanetaryWinds(context.grid, context.recipe.axialTiltDegrees);
    context.scratch.windX = winds.x;
    context.scratch.windY = winds.y;
    const circulation = createOceanCirculation(context.grid, context.fields.waterDepth, winds);
    context.scratch.currentX = circulation.x;
    context.scratch.currentY = circulation.y;
    context.fields.oceanCurrent = circulation.speed;
    context.fields.upwelling = circulation.upwelling;
    report(0.66, "Surface gyres and upwelling zones resolved");
    context.fields.seaSurfaceTemperature = createSeaSurfaceTemperature(
      context.grid,
      context.fields.waterDepth,
      circulation,
      context.constraints,
      context.recipe.solarConstantWm2 / 1361,
      context.recipe.oceanHeatTransport
    );
    report(1, "Sea-surface heat transport equilibrated");
  }
};
