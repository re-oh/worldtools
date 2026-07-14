import { calculateFieldStats } from "../../lib/field/field";
import type { BulkFieldMath } from "../../lib/field/BulkFieldMath";
import { createGridGeometry } from "../../lib/field/grid";
import { hash32 } from "../../lib/random/deterministic";
import { checksumFields } from "../checksum";
import { createConstraintSet, validateConstraintSet, type ConstraintSet } from "../constraints";
import { FIELD_IDS, type PackedWorld, type WorldStageStatus } from "../model";
import { normalizeRecipe, WORLD_VERSION, type WorldRecipe } from "../recipe";
import { climateStage } from "./climate/stage";
import { createEngineContext, type ProgressReporter } from "./context";
import { cryosphereStage } from "./cryosphere/stage";
import { ecologyStage } from "./ecology/stage";
import { geologyStage } from "./geology/stage";
import { hydrologyStage } from "./hydrology/stage";
import { oceanStage } from "./ocean/stage";
import { resourcesStage } from "./resources/stage";
import { soilsStage } from "./soils/stage";
import { executeStage, type WorldEngineStage } from "./stage";
import { tectonicsStage } from "./tectonics/stage";
import { terrainStage } from "./terrain/stage";

export const WORLD_ENGINE_STAGES: readonly WorldEngineStage[] = [
  geologyStage,
  tectonicsStage,
  terrainStage,
  oceanStage,
  climateStage,
  cryosphereStage,
  hydrologyStage,
  soilsStage,
  ecologyStage,
  resourcesStage
];

export interface GenerationOptions {
  constraints?: ConstraintSet;
  report?: ProgressReporter;
  now?: () => Date;
  bulkMath?: BulkFieldMath;
}

export function generatePackedWorld(recipeInput: Partial<WorldRecipe> = {}, options: GenerationOptions = {}): PackedWorld {
  const recipe = normalizeRecipe(recipeInput);
  const grid = createGridGeometry(recipe.width, recipe.height, recipe.radiusKm);
  const constraints = options.constraints ?? createConstraintSet(grid.cellCount);
  validateConstraintSet(constraints, grid);
  const context = createEngineContext(recipe, constraints, grid, options.bulkMath);
  const started = performance.now();
  const stages: WorldStageStatus[] = [];
  for (const stage of WORLD_ENGINE_STAGES) {
    const status = executeStage(stage, context, options.report);
    stages.push(status);
    if (status.state === "failed") throw new Error(`${status.label} failed: ${status.detail}`);
  }
  const stats = Object.fromEntries(FIELD_IDS.map((id) => [id, calculateFieldStats(context.fields[id])])) as PackedWorld["stats"];
  const checksum = checksumFields(context.fields, hash32(recipe.seed, recipe.width, recipe.height, recipe.plateCount));
  return {
    version: WORLD_VERSION,
    id: `bombo-${checksum}`,
    name: `World ${recipe.seed.toString(16).toUpperCase().padStart(8, "0")}`,
    recipe,
    generatedAt: (options.now ?? (() => new Date()))().toISOString(),
    generationMs: performance.now() - started,
    fields: context.fields,
    stats,
    downstream: context.downstream,
    basin: context.basin,
    plates: context.plates,
    deposits: context.deposits,
    stages,
    checksum,
    notes: context.notes
  };
}
