import type { WorldStageStatus, WorldStageId } from "../model";
import type { ProgressReporter, WorldEngineContext } from "./context";

export interface WorldEngineStage {
  id: WorldStageId;
  label: string;
  detail: string;
  run(context: WorldEngineContext, report: (progress: number, detail?: string) => void): void;
}

export function executeStage(stage: WorldEngineStage, context: WorldEngineContext, report?: ProgressReporter): WorldStageStatus {
  const start = performance.now();
  report?.({ stageId: stage.id, label: stage.label, progress: 0, detail: stage.detail });
  try {
    stage.run(context, (progress, detail = stage.detail) => {
      report?.({ stageId: stage.id, label: stage.label, progress: Math.max(0, Math.min(1, progress)), detail });
    });
    const durationMs = performance.now() - start;
    report?.({ stageId: stage.id, label: stage.label, progress: 1, detail: "Complete" });
    return { id: stage.id, label: stage.label, state: "complete", progress: 1, durationMs, detail: stage.detail };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { id: stage.id, label: stage.label, state: "failed", progress: 1, durationMs: performance.now() - start, detail };
  }
}
