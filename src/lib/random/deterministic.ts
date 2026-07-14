import { clamp, lerp } from "../math/scalar";

export function hash32(a: number, b = 0, c = 0, d = 0): number {
  let value = (Math.imul(a | 0, 0x9e3779b1) ^ Math.imul(b | 0, 0x85ebca77) ^ Math.imul(c | 0, 0xc2b2ae3d) ^ (d | 0)) >>> 0;
  value ^= value >>> 16;
  value = Math.imul(value, 0x7feb352d) >>> 0;
  value ^= value >>> 15;
  value = Math.imul(value, 0x846ca68b) >>> 0;
  return (value ^ (value >>> 16)) >>> 0;
}

export function random01(seed: number, stream: number, index: number, salt = 0): number {
  return hash32(seed, stream, index, salt) / 0x100000000;
}

export function randomSigned(seed: number, stream: number, index: number, salt = 0): number {
  return random01(seed, stream, index, salt) * 2 - 1;
}

export function hashText(text: string): string {
  let hash = 2166136261;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

export function valueNoise3(x: number, y: number, z: number, seed: number): number {
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const z0 = Math.floor(z);
  const tx = fade(x - x0);
  const ty = fade(y - y0);
  const tz = fade(z - z0);
  const sample = (dx: number, dy: number, dz: number) => randomSigned(seed, x0 + dx, y0 + dy, z0 + dz);
  const x00 = lerp(sample(0, 0, 0), sample(1, 0, 0), tx);
  const x10 = lerp(sample(0, 1, 0), sample(1, 1, 0), tx);
  const x01 = lerp(sample(0, 0, 1), sample(1, 0, 1), tx);
  const x11 = lerp(sample(0, 1, 1), sample(1, 1, 1), tx);
  return lerp(lerp(x00, x10, ty), lerp(x01, x11, ty), tz);
}

export function fractalNoise3(x: number, y: number, z: number, seed: number, scale: number, octaves: number, persistence = 0.5): number {
  let amplitude = 1;
  let frequency = scale;
  let total = 0;
  let weight = 0;
  for (let octave = 0; octave < octaves; octave += 1) {
    const offset = random01(seed, 991, octave) * 37;
    total += valueNoise3(x * frequency + offset, y * frequency - offset * 0.73, z * frequency + offset * 0.41, seed + octave * 1013) * amplitude;
    weight += amplitude;
    amplitude *= persistence;
    frequency *= 2.03;
  }
  return clamp(total / Math.max(weight, 1e-9), -1, 1);
}

export function ridgedNoise3(x: number, y: number, z: number, seed: number, scale: number, octaves: number): number {
  return 1 - Math.abs(fractalNoise3(x, y, z, seed, scale, octaves));
}

function fade(value: number): number {
  return value * value * value * (value * (value * 6 - 15) + 10);
}
