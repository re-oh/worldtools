import type { FieldId, PackedWorld } from "../world/model";

export interface RendererBackend {
  readonly name: "WebGPU";
  readonly detail: string;
}

export interface FieldRenderer {
  readonly backend: RendererBackend;
  readonly lost: Promise<string>;
  setWorld(world: PackedWorld): void;
  setLayer(layer: FieldId): void;
  setSelection(cell: number | null): void;
  resize(): boolean;
  render(): void;
  zoomAt(clientX: number, clientY: number, deltaY: number): void;
  panBy(deltaClientX: number, deltaClientY: number): void;
  resetView(): void;
  cellAt(clientX: number, clientY: number): number | null;
  pixelsForDistanceKm(distanceKm: number): number;
  dispose(): void;
}
