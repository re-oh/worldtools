import { describe, expect, it } from "vitest";
import { generatePackedWorld } from "../src/world/engine/pipeline";
import { DEFAULT_RECIPE, recipeForPreset } from "../src/world/recipe";

describe("draft world quality", () => {
  it("retains diverse physical and resource regimes at the shipped preset", () => {
    const world = generatePackedWorld({ ...recipeForPreset(DEFAULT_RECIPE, "draft"), erosionCycles: 12 });
    const cells = world.recipe.width * world.recipe.height;
    const land = Array.from(world.fields.waterDepth).filter((value) => value === 0).length;
    const biomes = new Set(Array.from(world.fields.biome).map(Math.round));
    const types = new Set(world.deposits.map((deposit) => deposit.typeId));
    const classes = new Set(world.deposits.map((deposit) => deposit.resourceClass));
    expect(land / cells).toBeGreaterThan(0.2);
    expect(land / cells).toBeLessThan(0.7);
    expect(biomes.size).toBeGreaterThanOrEqual(7);
    expect(types.size).toBeGreaterThanOrEqual(8);
    expect(classes.size).toBeGreaterThanOrEqual(3);
    expect([...types].some((type) => type.includes("bauxite"))).toBe(true);
    expect([...types].some((type) => type.includes("coal"))).toBe(true);
    expect([...types].some((type) => type.includes("oil") || type.includes("gas"))).toBe(true);
    expect(world.stats.delta.maximum).toBeGreaterThan(0);
    expect(world.stats.glacialErosion.maximum).toBeGreaterThan(0);
  });
});
