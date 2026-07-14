export function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}

export function saturate(value: number): number {
  return clamp(value, 0, 1);
}

export function lerp(a: number, b: number, amount: number): number {
  return a + (b - a) * amount;
}

export function inverseLerp(a: number, b: number, value: number): number {
  return Math.abs(b - a) < 1e-12 ? 0 : (value - a) / (b - a);
}

export function smoothstep(edge0: number, edge1: number, value: number): number {
  const t = saturate(inverseLerp(edge0, edge1, value));
  return t * t * (3 - 2 * t);
}

export function bell(value: number, center: number, width: number): number {
  const normalized = (value - center) / Math.max(width, 1e-9);
  return Math.exp(-(normalized * normalized));
}

export function signedPow(value: number, exponent: number): number {
  return Math.sign(value) * Math.pow(Math.abs(value), exponent);
}

export function quantize(value: number, steps: number): number {
  return Math.round(value * steps) / steps;
}
