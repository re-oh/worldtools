import { clamp } from "./scalar";

export type Vec3 = readonly [number, number, number];

export function dot3(a: Vec3, b: Vec3): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

export function cross3(a: Vec3, b: Vec3): [number, number, number] {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

export function normalize3(vector: Vec3): [number, number, number] {
  const length = Math.hypot(vector[0], vector[1], vector[2]);
  if (!Number.isFinite(length) || length < 1e-12) return [0, 0, 1];
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

export function angularDistance3(a: Vec3, b: Vec3): number {
  return Math.acos(clamp(dot3(a, b), -1, 1));
}

export function tangentToward3(from: Vec3, to: Vec3): [number, number, number] {
  const projection = dot3(to, from);
  return normalize3([to[0] - from[0] * projection, to[1] - from[1] * projection, to[2] - from[2] * projection]);
}
