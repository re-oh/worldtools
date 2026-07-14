import { describe, expect, it } from "vitest";
import { writeNeighbors } from "../src/lib/field/grid";
import { generatePackedWorld } from "../src/world/engine/pipeline";
import { FIELD_IDS, STAGE_IDS } from "../src/world/model";
import { DEFAULT_RECIPE } from "../src/world/recipe";

const recipe = { ...DEFAULT_RECIPE, width: 128, height: 64, plateCount: 10, erosionCycles: 6 };
let cached = generatePackedWorld(recipe, { now: () => new Date("2026-01-01T00:00:00.000Z") });

describe("0.3 packed world engine", () => {
  it("runs every stage and produces finite, non-constant physical fields", () => {
    expect(cached.stages.map((stage) => stage.id)).toEqual(STAGE_IDS);
    expect(cached.stages.every((stage) => stage.state === "complete")).toBe(true);
    for (const id of FIELD_IDS) {
      expect(cached.fields[id]).toHaveLength(recipe.width * recipe.height);
      expect(Array.from(cached.fields[id]).every(Number.isFinite)).toBe(true);
    }
    expect(cached.stats.elevation.maximum).toBeGreaterThan(1000);
    expect(cached.stats.elevation.minimum).toBeLessThan(-2000);
    expect(cached.stats.precipitation.maximum).toBeGreaterThan(cached.stats.precipitation.minimum * 2);
    expect(cached.stats.vegetationCover.maximum).toBeGreaterThan(0.2);
    expect(cached.stats.resourcePotential.maximum).toBeGreaterThan(0.3);
  });

  it("routes every land cell through neighboring cells without cycles", () => {
    const neighbors = new Int32Array(8);
    for (let start = 0; start < cached.downstream.length; start += 1) {
      if (cached.fields.waterDepth[start] > 0) continue;
      let cell = start;
      const visited = new Set<number>();
      while (cached.downstream[cell] >= 0) {
        expect(visited.has(cell)).toBe(false);
        visited.add(cell);
        const receiver = cached.downstream[cell];
        const count = writeNeighbors(cell, { width: recipe.width, height: recipe.height, cellCount: recipe.width * recipe.height, radiusKm: recipe.radiusKm }, neighbors);
        expect(Array.from(neighbors.slice(0, count))).toContain(receiver);
        cell = receiver;
      }
      expect(cached.fields.waterDepth[cell]).toBeGreaterThan(0);
    }
  });

  it("is deterministic and changes when the seed changes", () => {
    const replay = generatePackedWorld(recipe, { now: () => new Date("2030-01-01T00:00:00.000Z") });
    const changed = generatePackedWorld({ ...recipe, seed: recipe.seed + 1 });
    expect(replay.checksum).toBe(cached.checksum);
    expect(changed.checksum).not.toBe(cached.checksum);
    cached = replay;
  });

  it("emits process-attributed natural resource occurrences", () => {
    expect(cached.deposits.length).toBeGreaterThan(4);
    expect(new Set(cached.deposits.map((deposit) => deposit.resourceClass)).size).toBeGreaterThan(1);
    expect(cached.deposits.every((deposit) => deposit.formation.length > 30 && deposit.host.length > 3 && deposit.setting.length > 3)).toBe(true);
  });
});
