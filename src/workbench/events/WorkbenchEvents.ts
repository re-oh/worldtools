import type { InteractionMode } from "../interaction/MapInteractionController";
import type { PaintSign } from "../editing/ConstraintPainter";

export interface WorkbenchEventHandlers {
  onCommand(command: string): void;
  onMode(mode: InteractionMode): void;
  onPaintSign(sign: PaintSign): void;
  onLeftTab(tab: string): void;
  onInspectorTab(tab: string): void;
}

export class WorkbenchEvents {
  constructor(private readonly root: HTMLElement, private readonly handlers: WorkbenchEventHandlers) {
    root.addEventListener("click", this.onClick);
    window.addEventListener("keydown", this.onKeyDown);
  }

  dispose(): void {
    this.root.removeEventListener("click", this.onClick);
    window.removeEventListener("keydown", this.onKeyDown);
  }

  private readonly onClick = (event: MouseEvent): void => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button");
    if (!button) return;
    if (button.dataset.mode) this.handlers.onMode(button.dataset.mode as InteractionMode);
    if (button.dataset.paintSign !== undefined) this.handlers.onPaintSign(Number(button.dataset.paintSign) as PaintSign);
    if (button.dataset.leftTab) this.handlers.onLeftTab(button.dataset.leftTab);
    if (button.dataset.inspectorTab) this.handlers.onInspectorTab(button.dataset.inspectorTab);
    if (button.dataset.action) this.handlers.onCommand(button.dataset.action);
  };

  private readonly onKeyDown = (event: KeyboardEvent): void => {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
    if (event.ctrlKey && event.key.toLowerCase() === "z") {
      event.preventDefault();
      this.handlers.onCommand(event.shiftKey ? "redo" : "undo");
    } else if (event.ctrlKey && event.key.toLowerCase() === "y") {
      event.preventDefault();
      this.handlers.onCommand("redo");
    } else if (event.key === "1") this.handlers.onMode("inspect");
    else if (event.key === "2") this.handlers.onMode("pan");
    else if (event.key === "3") this.handlers.onMode("paint");
  };
}
