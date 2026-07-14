export const BLUE_NOISE_SIZE = 32;

const DEFAULT_SEED = 0x6d2b79f5;

/**
 * Builds a deterministic progressive farthest-point rank map on a torus.
 * Thresholding any early portion keeps samples separated, which makes this a
 * useful compact blue-noise dither without shipping a large opaque asset.
 */
export function buildBlueNoiseRankMap(size = BLUE_NOISE_SIZE, seed = DEFAULT_SEED): Uint8Array {
  if (!Number.isInteger(size) || size < 2) throw new RangeError("Blue-noise size must be an integer greater than one.");

  const count = size * size;
  const selected = new Uint8Array(count);
  const rankByPixel = new Uint32Array(count);
  const nearestDistance = new Float64Array(count);
  nearestDistance.fill(Number.POSITIVE_INFINITY);

  let current = hash32(seed) % count;
  for (let rank = 0; rank < count; rank += 1) {
    selected[current] = 1;
    rankByPixel[current] = rank;

    const selectedX = current % size;
    const selectedY = Math.floor(current / size);
    let next = -1;
    let greatestDistance = -1;
    let bestTieBreak = -1;

    for (let pixel = 0; pixel < count; pixel += 1) {
      if (selected[pixel] !== 0) continue;
      const x = pixel % size;
      const y = Math.floor(pixel / size);
      const dx = toroidalDistance(x, selectedX, size);
      const dy = toroidalDistance(y, selectedY, size);
      nearestDistance[pixel] = Math.min(nearestDistance[pixel], dx * dx + dy * dy);

      const tieBreak = hash32(pixel ^ seed ^ Math.imul(rank + 1, 0x9e3779b9));
      if (nearestDistance[pixel] > greatestDistance || (nearestDistance[pixel] === greatestDistance && tieBreak > bestTieBreak)) {
        greatestDistance = nearestDistance[pixel];
        bestTieBreak = tieBreak;
        next = pixel;
      }
    }

    if (next >= 0) current = next;
  }

  const output = new Uint8Array(count);
  for (let pixel = 0; pixel < count; pixel += 1) {
    output[pixel] = Math.min(255, Math.floor(((rankByPixel[pixel] + 0.5) * 256) / count));
  }
  return output;
}

export interface PaddedTextureRows {
  readonly data: Uint8Array;
  readonly bytesPerRow: number;
}

export function padTextureRows(source: Uint8Array, width: number, height: number, alignment = 256): PaddedTextureRows {
  if (source.length !== width * height) throw new RangeError("Texture data does not match its dimensions.");
  const bytesPerRow = Math.ceil(width / alignment) * alignment;
  if (bytesPerRow === width) return { data: source, bytesPerRow };

  const data = new Uint8Array(bytesPerRow * height);
  for (let row = 0; row < height; row += 1) {
    data.set(source.subarray(row * width, (row + 1) * width), row * bytesPerRow);
  }
  return { data, bytesPerRow };
}

function toroidalDistance(a: number, b: number, size: number): number {
  const direct = Math.abs(a - b);
  return Math.min(direct, size - direct);
}

function hash32(value: number): number {
  let hash = value >>> 0;
  hash ^= hash >>> 16;
  hash = Math.imul(hash, 0x7feb352d);
  hash ^= hash >>> 15;
  hash = Math.imul(hash, 0x846ca68b);
  return (hash ^ (hash >>> 16)) >>> 0;
}
