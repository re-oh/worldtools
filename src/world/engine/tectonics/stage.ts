import { saturate } from "../../../lib/math/scalar";
import type { WorldEngineStage } from "../stage";
import { createBoundaryFields, rejuvenateOceanCrust } from "./boundaries";
import { createHotspotField } from "./hotspots";
import { classifyProvinces } from "./provinces";

export const tectonicsStage: WorldEngineStage = {
  id: "tectonics",
  label: "Tectonic history",
  detail: "Resolving plate motion, deformation belts, magmatism, and basins",
  run(context, report) {
    const fields = context.fields;
    const boundary = createBoundaryFields(
      context.recipe,
      context.grid,
      fields.plateId,
      fields.continentalness,
      context.plates,
      context.constraints
    );
    fields.convergence = boundary.convergence;
    fields.divergence = boundary.divergence;
    fields.shear = boundary.shear;
    fields.uplift = boundary.uplift;
    report(0.54, "Deformation zones diffused beyond one-cell plate contacts");

    const hotspots = createHotspotField(context.recipe, context.grid);
    for (let index = 0; index < context.grid.cellCount; index += 1) {
      fields.volcanism[index] = saturate(Math.max(boundary.arcVolcanism[index] * context.recipe.volcanism, hotspots[index]));
    }
    rejuvenateOceanCrust(context.grid, fields.divergence, fields.continentalness, fields.crustAge, context.recipe.tectonicActivity);
    report(0.74, "Ridge ages and hotspot tracks resolved");

    const provinces = classifyProvinces(
      context.recipe,
      context.grid,
      fields.continentalness,
      fields.crustAge,
      fields.convergence,
      fields.divergence,
      fields.shear,
      fields.uplift,
      fields.volcanism
    );
    fields.sedimentaryBasin = provinces.sedimentaryBasin;
    fields.carbonatePlatform = provinces.carbonatePlatform;
    fields.lithology = provinces.lithology;
    report(1, "Rift, foreland, back-arc, and stable platform provinces classified");
  }
};
