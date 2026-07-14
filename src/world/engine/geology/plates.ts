import { MinHeap } from "../../../lib/collections/MinHeap";
import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";
import { random01, randomSigned } from "../../../lib/random/deterministic";
import { normalize3, type Vec3 } from "../../../lib/math/vector";
import type { PlateModel } from "../../model";
import type { WorldRecipe } from "../../recipe";

interface GrowthEntry {
  cell: number;
  plate: number;
  cost: number;
}

export interface PlatePartition {
  plates: PlateModel[];
  plateIds: Float32Array;
}

export function createPlatePartition(recipe: WorldRecipe, grid: GridGeometry): PlatePartition {
  const seeds = chooseSeparatedSeeds(recipe, grid);
  const plates = seeds.map((seedCell, id) => createPlate(recipe, grid, seedCell, id));
  const assigned = new Int32Array(grid.cellCount).fill(-1);
  const plateIds = new Float32Array(grid.cellCount);
  const frontier = new MinHeap<GrowthEntry>((a, b) => a.cost - b.cost || a.cell - b.cell || a.plate - b.plate);
  const neighbors = new Int32Array(8);

  for (const plate of plates) {
    assigned[plate.seedCell] = plate.id;
    plateIds[plate.seedCell] = plate.id;
    enqueueNeighbors(plate.seedCell, plate, 0, assigned, grid, neighbors, frontier, recipe.seed);
  }

  let remaining = grid.cellCount - plates.length;
  while (remaining > 0) {
    const entry = frontier.pop();
    if (!entry) throw new Error("Plate growth frontier exhausted before the sphere was covered.");
    if (assigned[entry.cell] >= 0) continue;
    assigned[entry.cell] = entry.plate;
    plateIds[entry.cell] = entry.plate;
    remaining -= 1;
    enqueueNeighbors(entry.cell, plates[entry.plate], entry.cost, assigned, grid, neighbors, frontier, recipe.seed);
  }

  return { plates, plateIds };
}

function chooseSeparatedSeeds(recipe: WorldRecipe, grid: GridGeometry): number[] {
  const seeds = [Math.floor(random01(recipe.seed, 101, 0) * grid.cellCount)];
  for (let plate = 1; plate < recipe.plateCount; plate += 1) {
    let best = -1;
    let bestScore = -1;
    const attempts = Math.min(192, 48 + recipe.plateCount * 4);
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      const candidate = Math.floor(random01(recipe.seed, 103 + plate, attempt) * grid.cellCount);
      if (seeds.includes(candidate)) continue;
      let minimumChord = Number.POSITIVE_INFINITY;
      for (const seed of seeds) {
        const dx = grid.x[candidate] - grid.x[seed];
        const dy = grid.y[candidate] - grid.y[seed];
        const dz = grid.z[candidate] - grid.z[seed];
        minimumChord = Math.min(minimumChord, dx * dx + dy * dy + dz * dz);
      }
      const score = minimumChord + random01(recipe.seed, 107, plate, attempt) * 0.004;
      if (score > bestScore) {
        best = candidate;
        bestScore = score;
      }
    }
    seeds.push(best >= 0 ? best : (seeds[0] + plate * 7919) % grid.cellCount);
  }
  return seeds;
}

function createPlate(recipe: WorldRecipe, grid: GridGeometry, seedCell: number, id: number): PlateModel {
  const pole = randomUnitVector(recipe.seed, 131, id);
  const direction = randomUnitVector(recipe.seed, 137, id);
  const maximumAge = Math.min(recipe.worldAgeGa * 1000 * 0.9, 3900);
  return {
    id,
    seedCell,
    continentalBias: randomSigned(recipe.seed, 139, id) * 0.75,
    ageMa: 180 + random01(recipe.seed, 149, id) * Math.max(80, maximumAge - 180),
    eulerPole: pole,
    angularVelocityDegreesMa: (0.12 + random01(recipe.seed, 151, id) * 0.88) * recipe.tectonicActivity,
    growthBias: 0.82 + random01(recipe.seed, 157, id) * 0.42,
    growthDirection: direction
  };
}

function enqueueNeighbors(
  cell: number,
  plate: PlateModel,
  parentCost: number,
  assigned: Int32Array,
  grid: GridGeometry,
  neighbors: Int32Array,
  frontier: MinHeap<GrowthEntry>,
  seed: number
): void {
  const count = writeNeighbors(cell, grid, neighbors);
  for (let offset = 0; offset < count; offset += 1) {
    const candidate = neighbors[offset];
    if (assigned[candidate] >= 0) continue;
    const directionScore =
      grid.x[candidate] * plate.growthDirection[0] +
      grid.y[candidate] * plate.growthDirection[1] +
      grid.z[candidate] * plate.growthDirection[2];
    const noise = random01(seed, 163 + plate.id, candidate);
    const step = Math.max(0.08, (0.64 + (1 - directionScore) * 0.2) * (1 - noise * 0.44) * plate.growthBias);
    frontier.push({ cell: candidate, plate: plate.id, cost: parentCost + step });
  }
}

function randomUnitVector(seed: number, stream: number, index: number): Vec3 {
  const azimuth = random01(seed, stream, index) * Math.PI * 2;
  const z = randomSigned(seed, stream + 1, index);
  const radius = Math.sqrt(Math.max(0, 1 - z * z));
  return normalize3([Math.cos(azimuth) * radius, Math.sin(azimuth) * radius, z]);
}
