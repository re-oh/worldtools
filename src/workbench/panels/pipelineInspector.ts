import type { WorldStageStatus } from "../../world/model";
import { emptyPanel } from "./panelDom";

export function renderPipelineInspector(container: HTMLElement, stages: readonly WorldStageStatus[]): void {
  if (stages.length === 0) {
    container.replaceChildren(emptyPanel("Pipeline has not run"));
    return;
  }
  const fragment = document.createDocumentFragment();
  stages.forEach((stage, index) => {
    const row = document.createElement("div");
    row.className = "stage-row";
    const order = document.createElement("span");
    order.className = "stage-index";
    order.textContent = String(index + 1).padStart(2, "0");
    const text = document.createElement("div");
    const name = document.createElement("strong");
    name.textContent = stage.label;
    const detail = document.createElement("small");
    detail.textContent = stage.state === "running" ? `${Math.round(stage.progress * 100)}% · ${stage.detail}` : stage.detail;
    text.append(name, detail);
    const duration = document.createElement("time");
    duration.textContent = stage.state === "complete" ? `${stage.durationMs.toFixed(0)} ms` : stage.state;
    row.append(order, text, duration);
    fragment.append(row);
  });
  container.replaceChildren(fragment);
}
