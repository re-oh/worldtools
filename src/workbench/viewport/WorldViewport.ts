import { createFieldRenderer } from "../../render/createRenderer";
import type { FieldRenderer } from "../../render/FieldRenderer";
import { layerDefinition } from "../../world/layerCatalog";
import { snapshotCell, type FieldId, type PackedWorld } from "../../world/model";
import type { ConstraintPainter } from "../editing/ConstraintPainter";
import { MapInteractionController, type InteractionMode } from "../interaction/MapInteractionController";
import type { WorkbenchShell } from "../shell/createShell";

export interface WorldViewportEvents {
  onSelect(cell: number): void;
  onPaintEnd(): void;
}

export class WorldViewport {
  private renderer: FieldRenderer | null = null;
  private interaction: MapInteractionController | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private world: PackedWorld | null = null;
  private activeLayer: FieldId = "elevation";
  private mode: InteractionMode = "inspect";
  private disposed = false;

  constructor(
    private readonly shell: WorkbenchShell,
    private readonly painter: ConstraintPainter,
    private readonly events: WorldViewportEvents
  ) {}

  async start(): Promise<void> {
    this.disposed = false;
    this.renderer = await createFieldRenderer(this.shell.canvas);
    const renderer = this.renderer;
    this.shell.backend.textContent = `${renderer.backend.name} | ${renderer.backend.detail}`;
    void renderer.lost.then((message) => {
      if (this.disposed || this.renderer !== renderer) return;
      this.shell.backend.textContent = `WebGPU lost | ${message}`;
      this.shell.generation.dataset.state = "error";
      this.shell.generation.hidden = false;
      this.shell.generationStage.textContent = "The WebGPU device was lost. Reload to create a new rendering device.";
      this.shell.progress.style.width = "0";
    });
    this.interaction = new MapInteractionController(this.shell.canvas, renderer, this.painter, {
      onSelect: (cell) => this.events.onSelect(cell),
      onHover: (cell, x, y) => this.showHover(cell, x, y),
      onPaintEnd: () => this.events.onPaintEnd(),
      onDragState: (dragging) => { this.shell.viewport.dataset.dragging = String(dragging); }
    });
    this.resizeObserver = new ResizeObserver(() => this.renderer?.resize());
    this.resizeObserver.observe(this.shell.viewport);
    renderer.resize();
    if (this.world) renderer.setWorld(this.world);
    renderer.setLayer(this.activeLayer);
    this.setMode(this.mode);
  }

  setWorld(world: PackedWorld): void {
    this.world = world;
    this.renderer?.setWorld(world);
  }

  setLayer(layer: FieldId): void {
    this.activeLayer = layer;
    this.renderer?.setLayer(layer);
    this.shell.activeLayerTitle.textContent = layerDefinition(layer).label;
  }

  setSelection(cell: number | null): void {
    this.renderer?.setSelection(cell);
  }

  setMode(mode: InteractionMode): void {
    this.mode = mode;
    this.interaction?.setMode(mode);
    this.shell.viewport.dataset.mode = mode;
    this.shell.paintBar.hidden = mode !== "paint";
    if (mode !== "paint") this.shell.brushCursor.hidden = true;
    for (const button of this.shell.root.querySelectorAll<HTMLButtonElement>("[data-mode]")) {
      button.setAttribute("aria-pressed", String(button.dataset.mode === mode));
    }
  }

  resetView(): void {
    this.renderer?.resetView();
  }

  zoomBy(delta: number): void {
    const rect = this.shell.canvas.getBoundingClientRect();
    this.renderer?.zoomAt(rect.left + rect.width / 2, rect.top + rect.height / 2, delta);
  }

  dispose(): void {
    this.disposed = true;
    this.interaction?.dispose();
    this.renderer?.dispose();
    this.resizeObserver?.disconnect();
  }

  private showHover(cell: number | null, clientX: number, clientY: number): void {
    if (!this.world || cell === null) {
      this.shell.coordinates.textContent = "Outside world";
      this.shell.brushCursor.hidden = true;
      return;
    }
    const snapshot = snapshotCell(this.world, cell);
    const definition = layerDefinition(this.activeLayer);
    this.shell.coordinates.textContent = `${snapshot.latitudeDegrees.toFixed(2)} deg, ${snapshot.longitudeDegrees.toFixed(2)} deg | ${definition.label} ${snapshot.values[this.activeLayer].toFixed(2)}`;
    if (this.mode !== "paint" || !this.renderer) return;
    const viewport = this.shell.viewport.getBoundingClientRect();
    const diameter = Math.max(8, this.renderer.pixelsForDistanceKm(Number(this.shell.brushRadius.value)) * 2);
    this.shell.brushCursor.style.left = `${clientX - viewport.left}px`;
    this.shell.brushCursor.style.top = `${clientY - viewport.top}px`;
    this.shell.brushCursor.style.width = `${diameter}px`;
    this.shell.brushCursor.style.height = `${diameter}px`;
    this.shell.brushCursor.hidden = false;
  }
}
