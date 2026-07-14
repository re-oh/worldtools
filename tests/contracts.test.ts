import { describe, expect, it } from "vitest";
import { constraintCount, createConstraintSet, resampleConstraintSet } from "../src/world/constraints";
import { parseProjectArchive } from "../src/world/projectArchive";
import { DEFAULT_RECIPE } from "../src/world/recipe";

describe("world project contracts", () => {
  it("resamples sparse manual constraints across supported resolutions", () => {
    const source = createConstraintSet(128 * 64);
    source.elevation[32 * 128 + 64] = 1200;
    source.moisture[20 * 128 + 8] = 0.6;
    const target = resampleConstraintSet(source, 128, 64, 256, 128);
    expect(target.elevation).toHaveLength(256 * 128);
    expect(Math.max(...target.elevation)).toBeGreaterThan(500);
    expect(constraintCount(target)).toBeGreaterThan(2);
  });

  it("restores a validated sparse project archive", () => {
    const recipe = { ...DEFAULT_RECIPE, width: 128, height: 64 };
    const restored = parseProjectArchive(JSON.stringify({
      format: "bombo-project",
      version: "0.3.0",
      name: "Test world",
      savedAt: "2026-01-02T00:00:00.000Z",
      expectedChecksum: "0123456789abcdef",
      recipe,
      constraints: [{ id: "elevation", entries: [[12, 450], [13, 50000]] }]
    }));
    expect(restored.recipe.width).toBe(128);
    expect(restored.constraints.elevation[12]).toBe(450);
    expect(restored.constraints.elevation[13]).toBe(4500);
    expect(restored.expectedChecksum).toBe("0123456789abcdef");
  });
});
