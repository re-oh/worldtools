import { propagateDistance } from "../shared/distance";
import type { WorldEngineStage } from "../stage";
import { createClimateDerivedFields } from "./derived";
import { createPrecipitation } from "./moisture";
import { createTemperatureFields } from "./temperature";

export const climateStage: WorldEngineStage = {
  id: "climate",
  label: "Climate and weather regimes",
  detail: "Transporting heat and moisture through planetary circulation and relief",
  run(context, report) {
    const coast = propagateDistance(context.grid, (cell) => context.fields.waterDepth[cell] > 0);
    const thermal = createTemperatureFields(
      context.recipe,
      context.grid,
      context.fields.elevation,
      context.fields.waterDepth,
      context.fields.seaSurfaceTemperature,
      coast,
      context.constraints
    );
    context.fields.temperature = thermal.mean;
    context.fields.temperatureSeasonality = thermal.seasonality;
    report(0.34, "Radiative, maritime, continental, and lapse-rate temperatures resolved");
    context.fields.precipitation = createPrecipitation(
      context.recipe,
      context.grid,
      context.fields.elevation,
      context.fields.waterDepth,
      thermal.mean,
      context.scratch.windX,
      context.scratch.windY,
      context.constraints
    );
    report(0.78, "Prevailing winds, rain shadows, and moisture recycling equilibrated");
    const derived = createClimateDerivedFields(
      context.grid,
      thermal.mean,
      thermal.seasonality,
      context.fields.precipitation,
      context.fields.seaSurfaceTemperature,
      context.scratch.windX,
      context.scratch.windY,
      coast
    );
    context.fields.aridity = derived.aridity;
    context.fields.windSpeed = derived.windSpeed;
    context.fields.storminess = derived.storminess;
    context.fields.freezeThaw = derived.freezeThaw;
    report(1, "Aridity, storm exposure, and freeze-thaw regimes complete");
  }
};
