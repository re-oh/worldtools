import { z } from "zod";
import { CONSTRAINT_IDS, decodeConstraintSet, encodeConstraintSet, type ConstraintSet } from "./constraints";
import type { PackedWorld } from "./model";
import { normalizeRecipe, WORLD_VERSION, WorldRecipeSchema, type WorldRecipe } from "./recipe";

const SparseConstraintSchema = z.object({
  id: z.enum(CONSTRAINT_IDS),
  entries: z.array(z.tuple([z.number().int().nonnegative(), z.number().finite()]))
});

const ProjectArchiveSchema = z.object({
  format: z.literal("bombo-project"),
  version: z.literal(WORLD_VERSION),
  name: z.string().trim().min(1).max(100),
  savedAt: z.string().datetime(),
  expectedChecksum: z.string().regex(/^[0-9a-f]{16}$/),
  recipe: WorldRecipeSchema,
  constraints: z.array(SparseConstraintSchema)
});

export type WorldProjectArchive = z.infer<typeof ProjectArchiveSchema>;

export interface RestoredProject {
  name: string;
  recipe: WorldRecipe;
  constraints: ConstraintSet;
  expectedChecksum: string;
}

export function createProjectArchive(world: PackedWorld, constraints: ConstraintSet, name = world.name): WorldProjectArchive {
  return {
    format: "bombo-project",
    version: WORLD_VERSION,
    name: name.trim().slice(0, 100) || world.name,
    savedAt: new Date().toISOString(),
    expectedChecksum: world.checksum,
    recipe: world.recipe,
    constraints: encodeConstraintSet(constraints)
  };
}

export function restoreProjectArchive(input: unknown): RestoredProject {
  const archive = ProjectArchiveSchema.parse(input);
  const recipe = normalizeRecipe(archive.recipe);
  return {
    name: archive.name,
    recipe,
    constraints: decodeConstraintSet(recipe.width * recipe.height, archive.constraints),
    expectedChecksum: archive.expectedChecksum
  };
}

export function parseProjectArchive(text: string): RestoredProject {
  return restoreProjectArchive(JSON.parse(text) as unknown);
}
