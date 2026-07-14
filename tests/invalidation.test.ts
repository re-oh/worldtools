import { describe, expect, it } from "vitest";
import { invalidateStages } from "../src/world/engine/invalidation";
import { STAGE_IDS, STAGE_LABELS, type WorldStageStatus } from "../src/world/model";

const completeStages = (): WorldStageStatus[] => STAGE_IDS.map((id) => ({
  id,
  label: STAGE_LABELS[id],
  state: "complete",
  progress: 1,
  durationMs: 1,
  detail: "Complete"
}));

describe("world stage invalidation", () => {
  it("preserves upstream stages and marks the edited stage and its dependents stale", () => {
    const stages = invalidateStages(completeStages(), "climate");

    expect(stages.find((stage) => stage.id === "ocean")?.state).toBe("complete");
    expect(stages.find((stage) => stage.id === "climate")?.state).toBe("stale");
    expect(stages.find((stage) => stage.id === "resources")?.state).toBe("stale");
  });
});
