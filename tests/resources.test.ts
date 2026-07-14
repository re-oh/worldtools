import { describe, expect, it } from "vitest";
import { DEPOSIT_TYPE_IDS } from "../src/world/model";
import { INDUSTRIAL_RESOURCE_MODELS } from "../src/world/engine/resources/models/industrial";
import { BEDROCK_MINERAL_MODELS } from "../src/world/engine/resources/models/minerals";
import { ORGANIC_RESOURCE_MODELS } from "../src/world/engine/resources/models/organic";
import { createPlacerModels } from "../src/world/engine/resources/transport";

describe("natural resource registry", () => {
  it("implements every public deposit type exactly once", () => {
    const placers = createPlacerModels(new Float32Array(1), new Float32Array(1));
    const models = [...BEDROCK_MINERAL_MODELS, ...INDUSTRIAL_RESOURCE_MODELS, ...ORGANIC_RESOURCE_MODELS, ...placers];
    const types = models.map((model) => model.typeId);
    expect([...types].sort()).toEqual([...DEPOSIT_TYPE_IDS].sort());
    expect(new Set(types).size).toBe(types.length);
  });

  it("gives each model process provenance and physical geometry", () => {
    const models = [...BEDROCK_MINERAL_MODELS, ...INDUSTRIAL_RESOURCE_MODELS, ...ORGANIC_RESOURCE_MODELS];
    expect(models.every((model) => model.formation.length > 60)).toBe(true);
    expect(models.every((model) => model.host && model.setting && model.commodities.length > 0)).toBe(true);
    expect(models.every((model) => model.depthMeters[1] >= model.depthMeters[0] && model.thicknessMeters[0] > 0)).toBe(true);
  });
});
