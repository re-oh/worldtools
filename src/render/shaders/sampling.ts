function cubicFieldSampler(functionName: string, fieldName: string): string {
  return /* wgsl */ `
fn ${functionName}(position: vec2f) -> f32 {
  let base = vec2i(floor(position));
  let amount = fract(position);
  let row0 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y - 1)], ${fieldName}[cellIndex(base.x, base.y - 1)],
    ${fieldName}[cellIndex(base.x + 1, base.y - 1)], ${fieldName}[cellIndex(base.x + 2, base.y - 1)]
  );
  let row1 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y)], ${fieldName}[cellIndex(base.x, base.y)],
    ${fieldName}[cellIndex(base.x + 1, base.y)], ${fieldName}[cellIndex(base.x + 2, base.y)]
  );
  let row2 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y + 1)], ${fieldName}[cellIndex(base.x, base.y + 1)],
    ${fieldName}[cellIndex(base.x + 1, base.y + 1)], ${fieldName}[cellIndex(base.x + 2, base.y + 1)]
  );
  let row3 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y + 2)], ${fieldName}[cellIndex(base.x, base.y + 2)],
    ${fieldName}[cellIndex(base.x + 1, base.y + 2)], ${fieldName}[cellIndex(base.x + 2, base.y + 2)]
  );
  let reconstructed = cubicInterpolate(vec4f(
    cubicInterpolate(row0, amount.x), cubicInterpolate(row1, amount.x),
    cubicInterpolate(row2, amount.x), cubicInterpolate(row3, amount.x)
  ), amount.y);
  let localMinimum = min(min(row1.y, row1.z), min(row2.y, row2.z));
  let localMaximum = max(max(row1.y, row1.z), max(row2.y, row2.z));
  return clamp(reconstructed, localMinimum, localMaximum);
}
`;
}

function cubicGradientSampler(functionName: string, fieldName: string): string {
  return /* wgsl */ `
fn ${functionName}(position: vec2f) -> vec2f {
  let base = vec2i(floor(position));
  let amount = fract(position);
  let row0 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y - 1)], ${fieldName}[cellIndex(base.x, base.y - 1)],
    ${fieldName}[cellIndex(base.x + 1, base.y - 1)], ${fieldName}[cellIndex(base.x + 2, base.y - 1)]
  );
  let row1 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y)], ${fieldName}[cellIndex(base.x, base.y)],
    ${fieldName}[cellIndex(base.x + 1, base.y)], ${fieldName}[cellIndex(base.x + 2, base.y)]
  );
  let row2 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y + 1)], ${fieldName}[cellIndex(base.x, base.y + 1)],
    ${fieldName}[cellIndex(base.x + 1, base.y + 1)], ${fieldName}[cellIndex(base.x + 2, base.y + 1)]
  );
  let row3 = vec4f(
    ${fieldName}[cellIndex(base.x - 1, base.y + 2)], ${fieldName}[cellIndex(base.x, base.y + 2)],
    ${fieldName}[cellIndex(base.x + 1, base.y + 2)], ${fieldName}[cellIndex(base.x + 2, base.y + 2)]
  );
  let rowValues = vec4f(
    cubicInterpolate(row0, amount.x), cubicInterpolate(row1, amount.x),
    cubicInterpolate(row2, amount.x), cubicInterpolate(row3, amount.x)
  );
  let rowDerivatives = vec4f(
    cubicDerivative(row0, amount.x), cubicDerivative(row1, amount.x),
    cubicDerivative(row2, amount.x), cubicDerivative(row3, amount.x)
  );
  return vec2f(cubicInterpolate(rowDerivatives, amount.y), cubicDerivative(rowValues, amount.y));
}
`;
}

export const FIELD_SAMPLING_SHADER = /* wgsl */ `
const BLUE_NOISE_SIZE: i32 = 32;
const BLUE_NOISE_FREQUENCY: f32 = 8.0;

fn cubicInterpolate(samples: vec4f, amount: f32) -> f32 {
  let a = -samples.x + 3.0 * samples.y - 3.0 * samples.z + samples.w;
  let b = 2.0 * samples.x - 5.0 * samples.y + 4.0 * samples.z - samples.w;
  let c = -samples.x + samples.z;
  return 0.5 * (((a * amount + b) * amount + c) * amount + 2.0 * samples.y);
}

fn cubicDerivative(samples: vec4f, amount: f32) -> f32 {
  let a = -samples.x + 3.0 * samples.y - 3.0 * samples.z + samples.w;
  let b = 2.0 * samples.x - 5.0 * samples.y + 4.0 * samples.z - samples.w;
  let c = -samples.x + samples.z;
  return 0.5 * ((3.0 * a * amount + 2.0 * b) * amount + c);
}

fn hashCoordinate(value: vec2i) -> u32 {
  var hash = bitcast<u32>(value.x) * 0x8da6b343u ^ bitcast<u32>(value.y) * 0xd8163841u;
  hash ^= hash >> 16u;
  hash *= 0x7feb352du;
  hash ^= hash >> 15u;
  return hash;
}

fn positiveModulo(value: i32, divisor: i32) -> i32 {
  return ((value % divisor) + divisor) % divisor;
}

fn blueNoiseDither(gridPosition: vec2f) -> f32 {
  let patternPosition = vec2i(floor(gridPosition * BLUE_NOISE_FREQUENCY));
  let patternTile = vec2i(floor(vec2f(patternPosition) / f32(BLUE_NOISE_SIZE)));
  let scramble = hashCoordinate(patternTile);
  let offset = vec2i(i32(scramble & 31u), i32((scramble >> 5u) & 31u));
  let coordinate = vec2i(
    positiveModulo(patternPosition.x + offset.x, BLUE_NOISE_SIZE),
    positiveModulo(patternPosition.y + offset.y, BLUE_NOISE_SIZE)
  );
  return textureLoad(blueNoiseRank, coordinate, 0).r - 0.5;
}

${cubicFieldSampler("sampleActiveContinuous", "activeField")}
${cubicFieldSampler("sampleFlowContinuous", "flow")}
${cubicGradientSampler("sampleElevationGradient", "elevation")}

fn sampleActive(mapUv: vec2f) -> f32 {
  let position = gridCoordinates(mapUv);
  if (uniforms.scaleMode > 1.5) {
    return activeField[cellIndex(i32(round(position.x)), i32(round(position.y)))];
  }
  return sampleActiveContinuous(position);
}

fn samplePalette(value: f32) -> vec3f {
  let normalized = normalizedValue(value);
  if (uniforms.scaleMode > 1.5) {
    let paletteIndex = clamp(i32(round(normalized * 255.0)), 0, 255);
    return textureLoad(palette, vec2i(paletteIndex, 0), 0).rgb;
  }
  return textureSampleLevel(palette, paletteSampler, vec2f(normalized, 0.5), 0.0).rgb;
}
`;
