export interface GridSpec {
  width: number;
  height: number;
  cellCount: number;
  radiusKm: number;
}

export interface GridGeometry extends GridSpec {
  latitudeRadians: Float32Array;
  cosLatitude: Float32Array;
  x: Float32Array;
  y: Float32Array;
  z: Float32Array;
  cellAreaKm2: number;
}

export function createGridGeometry(width: number, height: number, radiusKm: number): GridGeometry {
  const cellCount = width * height;
  const latitudeRadians = new Float32Array(height);
  const cosLatitude = new Float32Array(height);
  const x = new Float32Array(cellCount);
  const y = new Float32Array(cellCount);
  const z = new Float32Array(cellCount);
  for (let row = 0; row < height; row += 1) {
    const sinLatitude = -1 + ((row + 0.5) / height) * 2;
    const latitude = Math.asin(sinLatitude);
    const cosLatitudeValue = Math.cos(latitude);
    latitudeRadians[row] = latitude;
    cosLatitude[row] = cosLatitudeValue;
    for (let column = 0; column < width; column += 1) {
      const longitude = -Math.PI + ((column + 0.5) / width) * Math.PI * 2;
      const index = row * width + column;
      x[index] = cosLatitudeValue * Math.cos(longitude);
      y[index] = cosLatitudeValue * Math.sin(longitude);
      z[index] = sinLatitude;
    }
  }
  return {
    width,
    height,
    cellCount,
    radiusKm,
    latitudeRadians,
    cosLatitude,
    x,
    y,
    z,
    cellAreaKm2: (4 * Math.PI * radiusKm * radiusKm) / cellCount
  };
}

export function wrapColumn(column: number, width: number): number {
  return ((column % width) + width) % width;
}

export function writeNeighbors(index: number, grid: GridSpec, output: Int32Array): number {
  const row = Math.floor(index / grid.width);
  const column = index % grid.width;
  let count = 0;
  for (let rowOffset = -1; rowOffset <= 1; rowOffset += 1) {
    const nextRow = row + rowOffset;
    if (nextRow < 0 || nextRow >= grid.height) continue;
    for (let columnOffset = -1; columnOffset <= 1; columnOffset += 1) {
      if (rowOffset === 0 && columnOffset === 0) continue;
      output[count++] = nextRow * grid.width + wrapColumn(column + columnOffset, grid.width);
    }
  }
  return count;
}

export function greatCircleDistanceKm(a: number, b: number, grid: GridGeometry): number {
  const dot = grid.x[a] * grid.x[b] + grid.y[a] * grid.y[b] + grid.z[a] * grid.z[b];
  return Math.acos(Math.max(-1, Math.min(1, dot))) * grid.radiusKm;
}
