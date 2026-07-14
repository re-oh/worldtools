import type { FieldRenderer } from "../../render/FieldRenderer";
import type { ConstraintPainter } from "../editing/ConstraintPainter";

export type InteractionMode = "inspect" | "pan" | "paint";

export interface MapInteractionEvents {
  onSelect(cell: number): void;
  onHover(cell: number | null, clientX: number, clientY: number): void;
  onPaintEnd(): void;
  onDragState(dragging: boolean): void;
}

export class MapInteractionController {
  private mode: InteractionMode = "inspect";
  private pointerId: number | null = null;
  private lastX = 0;
  private lastY = 0;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly renderer: FieldRenderer,
    private readonly painter: ConstraintPainter,
    private readonly events: MapInteractionEvents
  ) {
    canvas.addEventListener("pointerdown", this.onPointerDown);
    canvas.addEventListener("pointermove", this.onPointerMove);
    canvas.addEventListener("pointerup", this.onPointerUp);
    canvas.addEventListener("pointercancel", this.onPointerUp);
    canvas.addEventListener("pointerleave", this.onPointerLeave);
    canvas.addEventListener("wheel", this.onWheel, { passive: false });
  }

  setMode(mode: InteractionMode): void {
    this.mode = mode;
  }

  dispose(): void {
    this.canvas.removeEventListener("pointerdown", this.onPointerDown);
    this.canvas.removeEventListener("pointermove", this.onPointerMove);
    this.canvas.removeEventListener("pointerup", this.onPointerUp);
    this.canvas.removeEventListener("pointercancel", this.onPointerUp);
    this.canvas.removeEventListener("pointerleave", this.onPointerLeave);
    this.canvas.removeEventListener("wheel", this.onWheel);
  }

  private readonly onPointerDown = (event: PointerEvent): void => {
    const cell = this.renderer.cellAt(event.clientX, event.clientY);
    if (cell === null) return;
    this.pointerId = event.pointerId;
    this.lastX = event.clientX;
    this.lastY = event.clientY;
    this.canvas.setPointerCapture(event.pointerId);
    this.events.onDragState(true);
    if (this.mode === "inspect") this.events.onSelect(cell);
    if (this.mode === "paint") this.painter.begin(cell);
  };

  private readonly onPointerMove = (event: PointerEvent): void => {
    const cell = this.renderer.cellAt(event.clientX, event.clientY);
    this.events.onHover(cell, event.clientX, event.clientY);
    if (event.pointerId !== this.pointerId) return;
    if (this.mode === "pan") this.renderer.panBy(event.clientX - this.lastX, event.clientY - this.lastY);
    if (this.mode === "paint" && cell !== null) this.painter.paint(cell);
    this.lastX = event.clientX;
    this.lastY = event.clientY;
  };

  private readonly onPointerUp = (event: PointerEvent): void => {
    if (event.pointerId !== this.pointerId) return;
    if (this.mode === "paint" && this.painter.end()) this.events.onPaintEnd();
    this.pointerId = null;
    this.events.onDragState(false);
  };

  private readonly onPointerLeave = (event: PointerEvent): void => {
    if (this.pointerId === null) this.events.onHover(null, event.clientX, event.clientY);
  };

  private readonly onWheel = (event: WheelEvent): void => {
    event.preventDefault();
    this.renderer.zoomAt(event.clientX, event.clientY, event.deltaY);
  };
}
