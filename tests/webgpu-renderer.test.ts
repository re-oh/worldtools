import { describe, expect, it } from "vitest";
import { createFieldRenderer } from "../src/render/createRenderer";
import { WORLD_RENDER_SHADER } from "../src/render/shaders/world";
import { BLUE_NOISE_SIZE, buildBlueNoiseRankMap, padTextureRows } from "../src/render/blueNoise";

describe("direct WebGPU renderer", () => {
  it("renders packed fields from storage buffers in one direct pipeline", () => {
    expect(WORLD_RENDER_SHADER).toContain("var<storage, read> activeField");
    expect(WORLD_RENDER_SHADER).toContain("var<storage, read> elevation");
    expect(WORLD_RENDER_SHADER).toContain("@vertex");
    expect(WORLD_RENDER_SHADER).toContain("@fragment");
    expect(WORLD_RENDER_SHADER).toContain("textureSampleLevel");
  });

  it("reconstructs continuous fields without exposing bilinear texel tents", () => {
    expect(WORLD_RENDER_SHADER).toContain("sampleActiveContinuous");
    expect(WORLD_RENDER_SHADER).toContain("cubicInterpolate");
    expect(WORLD_RENDER_SHADER).toContain("localMinimum");
    expect(WORLD_RENDER_SHADER).not.toContain("let a = mix(activeField");
    expect(WORLD_RENDER_SHADER).not.toContain("gridLine");
  });

  it("keeps categorical fields discrete and dithers only continuous output", () => {
    expect(WORLD_RENDER_SHADER).toContain("textureLoad(palette");
    expect(WORLD_RENDER_SHADER).toContain("blueNoiseDither");
    expect(WORLD_RENDER_SHADER).toContain("uniforms.scaleMode < 1.5");
  });

  it("composites physical overlays in the shader", () => {
    expect(WORLD_RENDER_SHADER).toContain("waterDepth");
    expect(WORLD_RENDER_SHADER).toContain("sampleFlowContinuous");
    expect(WORLD_RENDER_SHADER).toContain("plateId");
    expect(WORLD_RENDER_SHADER).toContain("useHillshade");
  });

  it("has no canvas or compatibility renderer path", () => {
    const factory = createFieldRenderer.toString();
    expect(factory).toContain("WebGpuFieldRenderer.create");
    expect(factory).not.toContain('getContext("2d")');
    expect(factory.toLowerCase()).not.toContain("fallback");
  });
});

describe("blue-noise rank texture", () => {
  it("is deterministic, distributed, and upload-aligned", () => {
    const first = buildBlueNoiseRankMap();
    const second = buildBlueNoiseRankMap();
    expect(first).toEqual(second);
    expect(first).toHaveLength(BLUE_NOISE_SIZE * BLUE_NOISE_SIZE);
    expect(Math.min(...first)).toBe(0);
    expect(Math.max(...first)).toBe(255);

    const padded = padTextureRows(first, BLUE_NOISE_SIZE, BLUE_NOISE_SIZE);
    expect(padded.bytesPerRow % 256).toBe(0);
    expect(padded.data).toHaveLength(padded.bytesPerRow * BLUE_NOISE_SIZE);
  });
});
