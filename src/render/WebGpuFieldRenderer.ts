import type { FieldId, PackedWorld } from "../world/model";
import { layerDefinition } from "../world/layerCatalog";
import type { FieldRenderer, RendererBackend } from "./FieldRenderer";
import { fitMapFrame, MapCamera } from "./MapCamera";
import { buildPaletteTexture } from "./palette";
import { BLUE_NOISE_SIZE, buildBlueNoiseRankMap, padTextureRows } from "./blueNoise";
import { WORLD_RENDER_SHADER } from "./shaders/world";

export class WebGpuFieldRenderer implements FieldRenderer {
  readonly backend: RendererBackend;
  readonly lost: Promise<string>;
  private readonly camera = new MapCamera();
  private world: PackedWorld | null = null;
  private layer: FieldId = "elevation";
  private selectedCell: number | null = null;
  private pipeline: GPURenderPipeline;
  private uniformBuffer: GPUBuffer;
  private paletteTexture: GPUTexture;
  private blueNoiseTexture: GPUTexture;
  private paletteSampler: GPUSampler;
  private bindGroup: GPUBindGroup | null = null;
  private fieldBuffers: GPUBuffer[] = [];

  private constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly context: GPUCanvasContext,
    private readonly device: GPUDevice,
    private readonly format: GPUTextureFormat,
    adapterLabel: string
  ) {
    this.backend = { name: "WebGPU", detail: adapterLabel };
    this.lost = device.lost.then((info) => info.message || `Device ${info.reason}`);
    const module = device.createShaderModule({ label: "Bombo world shader", code: WORLD_RENDER_SHADER });
    this.pipeline = device.createRenderPipeline({
      label: "Bombo world field pipeline",
      layout: "auto",
      vertex: { module, entryPoint: "vertexMain" },
      fragment: { module, entryPoint: "fragmentMain", targets: [{ format }] },
      primitive: { topology: "triangle-list" }
    });
    this.uniformBuffer = device.createBuffer({ label: "World uniforms", size: 64, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
    this.paletteTexture = device.createTexture({ label: "Layer palette", size: [256, 1], format: "rgba8unorm", usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST });
    this.blueNoiseTexture = device.createTexture({ label: "Stable blue-noise dither", size: [BLUE_NOISE_SIZE, BLUE_NOISE_SIZE], format: "r8unorm", usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST });
    const blueNoise = padTextureRows(buildBlueNoiseRankMap(), BLUE_NOISE_SIZE, BLUE_NOISE_SIZE);
    device.queue.writeTexture(
      { texture: this.blueNoiseTexture },
      blueNoise.data.buffer as ArrayBuffer,
      { bytesPerRow: blueNoise.bytesPerRow, rowsPerImage: BLUE_NOISE_SIZE },
      { width: BLUE_NOISE_SIZE, height: BLUE_NOISE_SIZE }
    );
    this.paletteSampler = device.createSampler({ magFilter: "linear", minFilter: "linear", addressModeU: "clamp-to-edge" });
    this.context.configure({ device, format, alphaMode: "opaque", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC });
  }

  static async create(canvas: HTMLCanvasElement): Promise<WebGpuFieldRenderer> {
    if (!navigator.gpu) throw new Error("WebGPU is unavailable in this browser.");
    const adapter = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
    if (!adapter) throw new Error("No compatible WebGPU adapter was found.");
    const device = await adapter.requestDevice();
    const context = canvas.getContext("webgpu");
    if (!context) throw new Error("The canvas could not create a WebGPU context.");
    const info = adapter.info;
    const label = info?.description || info?.vendor || "High-performance adapter";
    device.pushErrorScope("validation");
    let renderer: WebGpuFieldRenderer;
    try {
      renderer = new WebGpuFieldRenderer(canvas, context, device, navigator.gpu.getPreferredCanvasFormat(), label);
    } catch (error) {
      await device.popErrorScope().catch(() => null);
      device.destroy();
      throw error;
    }
    let validationError: GPUError | null;
    try {
      validationError = await device.popErrorScope();
    } catch (error) {
      renderer.dispose();
      throw error;
    }
    if (validationError) {
      renderer.dispose();
      throw new Error(`WebGPU world pipeline validation failed: ${validationError.message}`);
    }
    return renderer;
  }

  setWorld(world: PackedWorld): void {
    this.world = world;
    this.camera.reset();
    this.selectedCell = null;
    this.rebuildBuffers();
    this.setLayer(this.layer);
  }

  setLayer(layer: FieldId): void {
    this.layer = layer;
    if (!this.world || this.fieldBuffers.length === 0) return;
    const active = this.world.fields[layer];
    this.device.queue.writeBuffer(this.fieldBuffers[0], 0, active.buffer as ArrayBuffer, active.byteOffset, active.byteLength);
    const definition = layerDefinition(layer);
    const palette = buildPaletteTexture(definition.palette);
    this.device.queue.writeTexture(
      { texture: this.paletteTexture },
      palette.buffer as ArrayBuffer,
      { bytesPerRow: 1024 },
      { width: 256, height: 1 }
    );
    this.render();
  }

  setSelection(cell: number | null): void {
    this.selectedCell = cell;
    this.render();
  }

  resize(): boolean {
    const rect = this.canvas.getBoundingClientRect();
    const pixelRatio = Math.min(devicePixelRatio || 1, 2);
    const width = Math.max(1, Math.round(rect.width * pixelRatio));
    const height = Math.max(1, Math.round(rect.height * pixelRatio));
    if (this.canvas.width === width && this.canvas.height === height) return false;
    this.canvas.width = width;
    this.canvas.height = height;
    this.render();
    return true;
  }

  render(): void {
    if (!this.world || !this.bindGroup || this.canvas.width < 1 || this.canvas.height < 1) return;
    const definition = layerDefinition(this.layer);
    const range = definition.fixedRange ?? [this.world.stats[this.layer].minimum, this.world.stats[this.layer].maximum];
    const scaleMode = definition.scale === "log" ? 1 : definition.scale === "categorical" ? 2 : 0;
    const overlayMask = 1 + (["elevation", "flow", "runoff", "floodplain", "delta"].includes(this.layer) ? 2 : 0) + (["plateId", "convergence", "divergence", "shear", "uplift", "volcanism"].includes(this.layer) ? 4 : 0);
    const uniforms = new Float32Array([
      this.world.recipe.width, this.world.recipe.height,
      range[0], range[1],
      this.canvas.width, this.canvas.height,
      this.camera.center[0], this.camera.center[1],
      this.camera.zoom, scaleMode, Number(definition.hillshade), this.selectedCell ?? -1,
      overlayMask, 0, 0, 0
    ]);
    this.device.queue.writeBuffer(this.uniformBuffer, 0, uniforms);
    const encoder = this.device.createCommandEncoder({ label: "World render commands" });
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ view: this.context.getCurrentTexture().createView(), clearValue: { r: 0.027, g: 0.043, b: 0.047, a: 1 }, loadOp: "clear", storeOp: "store" }]
    });
    pass.setPipeline(this.pipeline);
    pass.setBindGroup(0, this.bindGroup);
    pass.draw(3);
    pass.end();
    this.device.queue.submit([encoder.finish()]);
  }

  zoomAt(clientX: number, clientY: number, deltaY: number): void {
    this.camera.zoomAt(this.canvas, clientX, clientY, deltaY);
    this.render();
  }

  panBy(deltaClientX: number, deltaClientY: number): void {
    this.camera.panBy(this.canvas, deltaClientX, deltaClientY);
    this.render();
  }

  resetView(): void {
    this.camera.reset();
    this.render();
  }

  cellAt(clientX: number, clientY: number): number | null {
    if (!this.world) return null;
    const point = this.camera.worldAt(this.canvas, clientX, clientY);
    if (!point) return null;
    const column = Math.max(0, Math.min(this.world.recipe.width - 1, Math.floor(point[0] * this.world.recipe.width)));
    const row = Math.max(0, Math.min(this.world.recipe.height - 1, Math.floor((1 - point[1]) * this.world.recipe.height)));
    return row * this.world.recipe.width + column;
  }

  pixelsForDistanceKm(distanceKm: number): number {
    if (!this.world) return 0;
    const rect = this.canvas.getBoundingClientRect();
    const frame = fitMapFrame(rect.width, rect.height);
    return distanceKm / (2 * Math.PI * this.world.recipe.radiusKm) * frame.width * this.camera.zoom;
  }

  dispose(): void {
    for (const buffer of this.fieldBuffers) buffer.destroy();
    this.uniformBuffer.destroy();
    this.paletteTexture.destroy();
    this.blueNoiseTexture.destroy();
    this.device.destroy();
  }

  private rebuildBuffers(): void {
    for (const buffer of this.fieldBuffers) buffer.destroy();
    if (!this.world) return;
    const values = [this.world.fields[this.layer], this.world.fields.elevation, this.world.fields.waterDepth, this.world.fields.flow, this.world.fields.plateId];
    this.fieldBuffers = values.map((field, index) => {
      const buffer = this.device.createBuffer({ label: `World field ${index}`, size: Math.max(4, field.byteLength), usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
      this.device.queue.writeBuffer(buffer, 0, field.buffer as ArrayBuffer, field.byteOffset, field.byteLength);
      return buffer;
    });
    this.bindGroup = this.device.createBindGroup({
      label: "World field bindings",
      layout: this.pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: { buffer: this.uniformBuffer } },
        ...this.fieldBuffers.map((buffer, index) => ({ binding: index + 1, resource: { buffer } })),
        { binding: 6, resource: this.paletteTexture.createView() },
        { binding: 7, resource: this.paletteSampler },
        { binding: 8, resource: this.blueNoiseTexture.createView() }
      ]
    });
  }
}
