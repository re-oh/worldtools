import type { StageProgress } from "../../world/engine/context";
import { invalidateStages } from "../../world/engine/invalidation";
import { STAGE_IDS, STAGE_LABELS, type WorldStageId, type WorldStageStatus } from "../../world/model";
import { renderPipelineInspector } from "../panels/pipelineInspector";
import type { WorkbenchShell } from "../shell/createShell";

export class GenerationMonitor {
  private stages: WorldStageStatus[] = [];
  private failed = false;

  constructor(private readonly shell: WorkbenchShell) {}

  begin(): void {
    this.failed = false;
    this.stages = STAGE_IDS.map((id) => ({
      id,
      label: STAGE_LABELS[id],
      state: "pending",
      progress: 0,
      durationMs: 0,
      detail: "Pending"
    }));
    this.showProgress("Preparing deterministic fields", 0);
    this.renderPipeline();
  }

  update(progress: StageProgress): void {
    const stageIndex = STAGE_IDS.indexOf(progress.stageId);
    this.stages = this.stages.map((stage, index) => index < stageIndex
      ? { ...stage, state: "complete", progress: 1 }
      : index === stageIndex
        ? { ...stage, label: progress.label, state: "running", progress: progress.progress, detail: progress.detail }
        : stage);
    this.showProgress(progress.detail, (stageIndex + progress.progress) / STAGE_IDS.length);
    this.renderPipeline();
  }

  complete(stages: readonly WorldStageStatus[]): void {
    this.stages = stages.map((stage) => ({ ...stage }));
    this.renderPipeline();
  }

  invalidate(from: WorldStageId): void {
    if (this.stages.length === 0) return;
    this.stages = invalidateStages(this.stages, from);
    this.renderPipeline();
  }

  fail(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    const running = this.stages.findIndex((stage) => stage.state === "running");
    const pendingIndex = this.stages.findIndex((stage) => stage.state === "pending");
    const failedIndex = running >= 0 ? running : pendingIndex;
    if (failedIndex >= 0) {
      this.stages = this.stages.map((stage, index) => index === failedIndex
        ? { ...stage, state: "failed", progress: 0, detail: message }
        : stage);
    }
    this.failed = true;
    this.shell.generation.hidden = false;
    this.shell.generation.dataset.state = "error";
    this.shell.generationStage.textContent = message;
    this.shell.progress.style.width = "0";
    this.renderPipeline();
  }

  finish(): void {
    this.shell.generation.hidden = !this.failed;
    this.shell.progress.style.width = "0";
  }

  renderPipeline(): void {
    renderPipelineInspector(this.shell.pipelineInspector, this.stages);
  }

  private showProgress(detail: string, progress: number): void {
    this.shell.generation.hidden = false;
    this.shell.generation.dataset.state = "running";
    this.shell.generationStage.textContent = detail;
    this.shell.progress.style.width = `${Math.max(0, Math.min(1, progress)) * 100}%`;
  }
}
