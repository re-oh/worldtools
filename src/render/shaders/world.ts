import { FIELD_SAMPLING_SHADER } from "./sampling";

export const WORLD_RENDER_SHADER = /* wgsl */ `
struct Uniforms {
  grid: vec2f,
  valueRange: vec2f,
  canvas: vec2f,
  center: vec2f,
  zoom: f32,
  scaleMode: f32,
  useHillshade: f32,
  selectedCell: f32,
  overlayMask: f32,
  padding0: f32,
  padding1: f32,
  padding2: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> activeField: array<f32>;
@group(0) @binding(2) var<storage, read> elevation: array<f32>;
@group(0) @binding(3) var<storage, read> waterDepth: array<f32>;
@group(0) @binding(4) var<storage, read> flow: array<f32>;
@group(0) @binding(5) var<storage, read> plateId: array<f32>;
@group(0) @binding(6) var palette: texture_2d<f32>;
@group(0) @binding(7) var paletteSampler: sampler;
@group(0) @binding(8) var blueNoiseRank: texture_2d<f32>;

@vertex
fn vertexMain(@builtin(vertex_index) vertexIndex: u32) -> @builtin(position) vec4f {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4f(positions[vertexIndex], 0.0, 1.0);
}

fn cellIndex(column: i32, row: i32) -> u32 {
  let width = i32(uniforms.grid.x);
  let height = i32(uniforms.grid.y);
  let wrappedColumn = ((column % width) + width) % width;
  return u32(clamp(row, 0, height - 1) * width + wrappedColumn);
}

fn gridCoordinates(mapUv: vec2f) -> vec2f {
  return vec2f(mapUv.x * uniforms.grid.x - 0.5, (1.0 - mapUv.y) * uniforms.grid.y - 0.5);
}

fn nearestIndex(mapUv: vec2f) -> u32 {
  let gridPosition = gridCoordinates(mapUv);
  return cellIndex(i32(round(gridPosition.x)), i32(round(gridPosition.y)));
}

fn normalizedValue(value: f32) -> f32 {
  let span = max(0.000001, uniforms.valueRange.y - uniforms.valueRange.x);
  if (uniforms.scaleMode > 0.5 && uniforms.scaleMode < 1.5) {
    return clamp(log(1.0 + max(0.0, value - uniforms.valueRange.x)) / log(1.0 + span), 0.0, 1.0);
  }
  return clamp((value - uniforms.valueRange.x) / span, 0.0, 1.0);
}

${FIELD_SAMPLING_SHADER}

@fragment
fn fragmentMain(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let screen = position.xy / uniforms.canvas;
  let aspect = uniforms.canvas.x / max(1.0, uniforms.canvas.y);
  var frame = vec2f(1.0, 1.0);
  if (aspect > 2.0) { frame.x = 2.0 / aspect; } else { frame.y = aspect / 2.0; }
  let origin = (vec2f(1.0) - frame) * 0.5;
  let local = (screen - origin) / frame;
  if (any(local < vec2f(0.0)) || any(local > vec2f(1.0))) { return vec4f(0.027, 0.043, 0.047, 1.0); }
  let mapUv = (local - vec2f(0.5)) / uniforms.zoom + uniforms.center;
  if (any(mapUv < vec2f(0.0)) || any(mapUv > vec2f(1.0))) { return vec4f(0.02, 0.03, 0.033, 1.0); }

  let cell = nearestIndex(mapUv);
  let gridPosition = gridCoordinates(mapUv);
  var color = samplePalette(sampleActive(mapUv));
  if (uniforms.useHillshade > 0.5) {
    let elevationGradient = sampleElevationGradient(gridPosition);
    let shade = clamp(0.98 - elevationGradient.x * 0.00015 + elevationGradient.y * 0.0001, 0.62, 1.28);
    color *= shade;
  }

  let overlays = u32(uniforms.overlayMask + 0.5);
  let column = i32(round(gridPosition.x));
  let row = i32(round(gridPosition.y));
  if ((overlays & 1u) != 0u) {
    let wet = waterDepth[cell] > 0.0;
    let coast = (waterDepth[cellIndex(column + 1, row)] > 0.0) != wet || (waterDepth[cellIndex(column - 1, row)] > 0.0) != wet || (waterDepth[cellIndex(column, row + 1)] > 0.0) != wet || (waterDepth[cellIndex(column, row - 1)] > 0.0) != wet;
    if (coast) { color = mix(color, vec3f(0.86, 0.9, 0.8), 0.58); }
  }
  if ((overlays & 2u) != 0u && waterDepth[cell] <= 0.0) {
    let river = smoothstep(max(0.48, 0.7 - log2(uniforms.zoom) * 0.045), 0.94, sampleFlowContinuous(gridPosition));
    color = mix(color, vec3f(0.15, 0.68, 0.78), river * 0.82);
  }
  if ((overlays & 4u) != 0u) {
    let plate = plateId[cell];
    let edge = plateId[cellIndex(column + 1, row)] != plate || plateId[cellIndex(column, row + 1)] != plate;
    if (edge) { color = mix(color, vec3f(0.95, 0.57, 0.28), 0.62); }
  }
  if (uniforms.selectedCell >= 0.0 && u32(uniforms.selectedCell) == cell) {
    let within = fract(gridPosition + vec2f(0.5));
    let border = min(min(within.x, within.y), min(1.0 - within.x, 1.0 - within.y));
    color = mix(color, vec3f(0.96, 0.98, 0.92), 1.0 - smoothstep(0.03, 0.13, border));
  }
  if (uniforms.scaleMode < 1.5) {
    color = clamp(color + vec3f(blueNoiseDither(gridPosition) / 255.0), vec3f(0.0), vec3f(1.0));
  }
  return vec4f(color, 1.0);
}
`;
