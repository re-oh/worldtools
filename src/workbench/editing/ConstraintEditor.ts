import { CommandHistory } from "../../lib/history/CommandHistory";
import {
  CONSTRAINT_DEFINITIONS,
  CONSTRAINT_IDS,
  type ConstraintId,
  type ConstraintSet
} from "../../world/constraints";
import type { WorldRecipe } from "../../world/recipe";
import type { WorldStageId } from "../../world/model";
import type { WorkbenchShell } from "../shell/createShell";
import { ConstraintPainter, type PaintSign } from "./ConstraintPainter";

export class ConstraintEditor {
  readonly painter: ConstraintPainter;
  private readonly history = new CommandHistory(100);

  constructor(private readonly shell: WorkbenchShell, onInvalidated: (stage: WorldStageId) => void) {
    this.painter = new ConstraintPainter(this.history, (constraint) => {
      onInvalidated(CONSTRAINT_DEFINITIONS[constraint].invalidates);
    });
    for (const id of CONSTRAINT_IDS) {
      shell.constraintSelect.add(new Option(CONSTRAINT_DEFINITIONS[id].label, id));
    }
    shell.constraintSelect.value = "elevation";
    shell.constraintSelect.addEventListener("change", this.updateSettings);
    shell.brushRadius.addEventListener("input", this.updateSettings);
    shell.brushStrength.addEventListener("input", this.updateSettings);
    this.updateSettings();
    this.refreshHistoryControls();
  }

  setWorld(recipe: WorldRecipe, constraints: ConstraintSet): void {
    this.painter.setWorld(recipe, constraints);
  }

  setSign(sign: PaintSign): void {
    for (const button of this.shell.root.querySelectorAll<HTMLButtonElement>("[data-paint-sign]")) {
      button.setAttribute("aria-pressed", String(Number(button.dataset.paintSign) === sign));
    }
    this.updateSettings();
  }

  undo(): void {
    this.history.undo();
    this.refreshHistoryControls();
  }

  redo(): void {
    this.history.redo();
    this.refreshHistoryControls();
  }

  clearHistory(): void {
    this.history.clear();
    this.refreshHistoryControls();
  }

  refreshHistoryControls(): void {
    const undo = this.shell.root.querySelector<HTMLButtonElement>("[data-action=undo]");
    const redo = this.shell.root.querySelector<HTMLButtonElement>("[data-action=redo]");
    if (undo) undo.disabled = !this.history.canUndo;
    if (redo) redo.disabled = !this.history.canRedo;
  }

  dispose(): void {
    this.shell.constraintSelect.removeEventListener("change", this.updateSettings);
    this.shell.brushRadius.removeEventListener("input", this.updateSettings);
    this.shell.brushStrength.removeEventListener("input", this.updateSettings);
  }

  private readonly updateSettings = (): void => {
    const signButton = this.shell.root.querySelector<HTMLButtonElement>("[data-paint-sign][aria-pressed=true]");
    this.painter.setSettings({
      constraint: this.shell.constraintSelect.value as ConstraintId,
      radiusKm: Number(this.shell.brushRadius.value),
      strength: Number(this.shell.brushStrength.value),
      sign: Number(signButton?.dataset.paintSign ?? 1) as PaintSign
    });
  };
}
