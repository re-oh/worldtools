import { vec2 } from "gl-matrix";

export interface PixelFrame {
  x: number;
  y: number;
  width: number;
  height: number;
}

export class MapCamera {
  readonly center = vec2.fromValues(0.5, 0.5);
  zoom = 1;

  reset(): void {
    vec2.set(this.center, 0.5, 0.5);
    this.zoom = 1;
  }

  panBy(canvas: HTMLCanvasElement, deltaClientX: number, deltaClientY: number): void {
    const rect = canvas.getBoundingClientRect();
    const frame = fitMapFrame(rect.width, rect.height);
    this.center[0] -= deltaClientX / Math.max(1, frame.width) / this.zoom;
    this.center[1] -= deltaClientY / Math.max(1, frame.height) / this.zoom;
    this.constrain();
  }

  zoomAt(canvas: HTMLCanvasElement, clientX: number, clientY: number, deltaY: number): void {
    const before = this.worldAt(canvas, clientX, clientY);
    this.zoom = Math.max(1, Math.min(32, this.zoom * Math.exp(-deltaY * 0.0014)));
    if (before) {
      const after = this.worldAt(canvas, clientX, clientY);
      if (after) vec2.add(this.center, this.center, vec2.fromValues(before[0] - after[0], before[1] - after[1]));
    }
    this.constrain();
  }

  worldAt(canvas: HTMLCanvasElement, clientX: number, clientY: number): readonly [number, number] | null {
    const rect = canvas.getBoundingClientRect();
    const frame = fitMapFrame(rect.width, rect.height);
    const x = clientX - rect.left;
    const y = clientY - rect.top;
    if (x < frame.x || x > frame.x + frame.width || y < frame.y || y > frame.y + frame.height) return null;
    const localX = (x - frame.x) / frame.width;
    const localY = (y - frame.y) / frame.height;
    const worldX = (localX - 0.5) / this.zoom + this.center[0];
    const worldY = (localY - 0.5) / this.zoom + this.center[1];
    return worldX >= 0 && worldX <= 1 && worldY >= 0 && worldY <= 1 ? [worldX, worldY] : null;
  }

  private constrain(): void {
    const half = 0.5 / this.zoom;
    this.center[0] = Math.max(half, Math.min(1 - half, this.center[0]));
    this.center[1] = Math.max(half, Math.min(1 - half, this.center[1]));
  }
}

export function fitMapFrame(width: number, height: number): PixelFrame {
  const aspect = width / Math.max(1, height);
  const frameWidth = aspect > 2 ? height * 2 : width;
  const frameHeight = aspect > 2 ? height : width / 2;
  return { x: (width - frameWidth) / 2, y: (height - frameHeight) / 2, width: frameWidth, height: frameHeight };
}
