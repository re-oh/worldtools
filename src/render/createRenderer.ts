import type { FieldRenderer } from "./FieldRenderer";
import { WebGpuFieldRenderer } from "./WebGpuFieldRenderer";

export async function createFieldRenderer(canvas: HTMLCanvasElement): Promise<FieldRenderer> {
  return WebGpuFieldRenderer.create(canvas);
}
