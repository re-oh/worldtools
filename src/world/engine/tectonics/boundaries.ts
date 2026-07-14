import { writeNeighbors, type GridGeometry } from "../../../lib/field/grid";
import { clamp, saturate } from "../../../lib/math/scalar";
import { cross3, dot3, normalize3, tangentToward3, type Vec3 } from "../../../lib/math/vector";
import type { ConstraintSet } from "../../constraints";
import type { PlateModel } from "../../model";
import type { WorldRecipe } from "../../recipe";
import { diffuseMaximum } from "./diffusion";

export interface BoundaryFields {
  convergence: Float32Array;
  divergence: Float32Array;
  shear: Float32Array;
  uplift: Float32Array;
  arcVolcanism: Float32Array;
}

export function createBoundaryFields(
  recipe: WorldRecipe,
  grid: GridGeometry,
  plateIds: Float32Array,
  continentalness: Float32Array,
  plates: PlateModel[],
  constraints: ConstraintSet
): BoundaryFields {
  const convergenceSource = new Float32Array(grid.cellCount);
  const divergenceSource = new Float32Array(grid.cellCount);
  const shearSource = new Float32Array(grid.cellCount);
  const upliftSource = new Float32Array(grid.cellCount);
  const volcanicSource = new Float32Array(grid.cellCount);

  for (let row = 0; row < grid.height; row += 1) {
    for (let column = 0; column < grid.width; column += 1) {
      const cell = row * grid.width + column;
      classifyContact(cell, row * grid.width + ((column + 1) % grid.width));
      if (row + 1 < grid.height) classifyContact(cell, (row + 1) * grid.width + column);
    }
  }

  const rings = Math.max(4, Math.min(14, Math.round(grid.width / 80)));
  return {
    convergence: diffuseMaximum(convergenceSource, grid, rings, 0.78),
    divergence: diffuseMaximum(divergenceSource, grid, Math.max(3, rings - 2), 0.75),
    shear: diffuseMaximum(shearSource, grid, Math.max(2, rings - 3), 0.7),
    uplift: diffuseMaximum(upliftSource, grid, rings + 2, 0.8),
    arcVolcanism: diffuseMaximum(volcanicSource, grid, Math.max(3, rings - 1), 0.73)
  };

  function classifyContact(a: number, b: number): void {
    const plateAId = Math.trunc(plateIds[a]);
    const plateBId = Math.trunc(plateIds[b]);
    if (plateAId === plateBId) return;
    const position = normalize3([grid.x[a] + grid.x[b], grid.y[a] + grid.y[b], grid.z[a] + grid.z[b]]);
    const relative = subtract(plateVelocity(plates[plateBId], position, grid.radiusKm), plateVelocity(plates[plateAId], position, grid.radiusKm));
    const normal = tangentToward3(position, [grid.x[b], grid.y[b], grid.z[b]]);
    const tangent = normalize3(cross3(position, normal));
    const convergenceRate = -dot3(relative, normal);
    const shearRate = Math.abs(dot3(relative, tangent));
    const edit = clamp(1 + (constraints.tectonicActivity[a] + constraints.tectonicActivity[b]) * 0.5, 0.15, 2);
    const convergence = saturate((convergenceRate / 6.5) * edit);
    const divergence = saturate((-convergenceRate / 5.5) * edit);
    const shear = saturate((shearRate / 6) * edit);
    const continentalA = saturate(continentalness[a] * 2);
    const continentalB = saturate(continentalness[b] * 2);
    const collision = Math.min(continentalA, continentalB);
    const subduction = Math.abs(continentalA - continentalB);

    convergenceSource[a] = Math.max(convergenceSource[a], convergence);
    convergenceSource[b] = Math.max(convergenceSource[b], convergence);
    divergenceSource[a] = Math.max(divergenceSource[a], divergence);
    divergenceSource[b] = Math.max(divergenceSource[b], divergence);
    shearSource[a] = Math.max(shearSource[a], shear);
    shearSource[b] = Math.max(shearSource[b], shear);
    const uplift = convergence * (0.55 + collision * 0.45 + subduction * 0.2) + shear * convergence * 0.18;
    upliftSource[a] = Math.max(upliftSource[a], uplift * (0.68 + continentalA * 0.32));
    upliftSource[b] = Math.max(upliftSource[b], uplift * (0.68 + continentalB * 0.32));
    volcanicSource[a] = Math.max(volcanicSource[a], divergence * 0.62 + convergence * (0.15 + continentalA * subduction * 0.8));
    volcanicSource[b] = Math.max(volcanicSource[b], divergence * 0.62 + convergence * (0.15 + continentalB * subduction * 0.8));
  }
}

export function rejuvenateOceanCrust(
  grid: GridGeometry,
  divergence: Float32Array,
  continentalness: Float32Array,
  crustAge: Float32Array,
  tectonicActivity: number
): void {
  const distance = new Int32Array(grid.cellCount).fill(-1);
  const queue = new Int32Array(grid.cellCount);
  let head = 0;
  let tail = 0;
  for (let index = 0; index < grid.cellCount; index += 1) {
    if (continentalness[index] < -0.12 && divergence[index] > 0.38) {
      distance[index] = 0;
      queue[tail++] = index;
    }
  }
  const neighbors = new Int32Array(8);
  while (head < tail) {
    const cell = queue[head++];
    const count = writeNeighbors(cell, grid, neighbors);
    for (let offset = 0; offset < count; offset += 1) {
      const next = neighbors[offset];
      if (distance[next] >= 0 || continentalness[next] >= 0) continue;
      distance[next] = distance[cell] + 1;
      queue[tail++] = next;
    }
  }
  const cellDistanceKm = Math.sqrt(grid.cellAreaKm2);
  const spreadingKmMa = 18 + tectonicActivity * 25;
  for (let index = 0; index < grid.cellCount; index += 1) {
    if (continentalness[index] >= 0 || distance[index] < 0) continue;
    crustAge[index] = clamp((distance[index] * cellDistanceKm) / spreadingKmMa, 0, 220);
  }
}

function plateVelocity(plate: PlateModel, position: Vec3, radiusKm: number): [number, number, number] {
  const tangent = cross3(plate.eulerPole, position);
  const centimetersPerYear = (plate.angularVelocityDegreesMa * Math.PI / 180) * radiusKm * 0.1;
  return [tangent[0] * centimetersPerYear, tangent[1] * centimetersPerYear, tangent[2] * centimetersPerYear];
}

function subtract(a: Vec3, b: Vec3): [number, number, number] {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}
