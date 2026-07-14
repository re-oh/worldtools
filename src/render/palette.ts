import type { PaletteStop } from "../world/layers/types";

export function buildPaletteTexture(stops: readonly PaletteStop[], size = 256): Uint8Array {
  const output = new Uint8Array(size * 4);
  const parsed = stops.map(([position, color]) => [position, parseHex(color)] as const);
  for (let index = 0; index < size; index += 1) {
    const amount = index / (size - 1);
    let upper = parsed.findIndex(([position]) => position >= amount);
    if (upper < 0) upper = parsed.length - 1;
    const lower = Math.max(0, upper - 1);
    const a = parsed[lower];
    const b = parsed[upper];
    const mix = a === b ? 0 : (amount - a[0]) / Math.max(1e-6, b[0] - a[0]);
    for (let channel = 0; channel < 3; channel += 1) output[index * 4 + channel] = Math.round(a[1][channel] + (b[1][channel] - a[1][channel]) * mix);
    output[index * 4 + 3] = 255;
  }
  return output;
}

export function paletteColor(stops: readonly PaletteStop[], amount: number): readonly [number, number, number] {
  const bytes = buildPaletteTexture(stops, 256);
  const index = Math.max(0, Math.min(255, Math.round(amount * 255))) * 4;
  return [bytes[index], bytes[index + 1], bytes[index + 2]];
}

function parseHex(value: string): readonly [number, number, number] {
  const hex = value.replace("#", "");
  if (hex.length !== 6) throw new Error(`Palette color ${value} must use six hexadecimal digits.`);
  return [Number.parseInt(hex.slice(0, 2), 16), Number.parseInt(hex.slice(2, 4), 16), Number.parseInt(hex.slice(4, 6), 16)];
}
