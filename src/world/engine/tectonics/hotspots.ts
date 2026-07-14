import { random01, randomSigned } from "../../../lib/random/deterministic";
import { normalize3 } from "../../../lib/math/vector";
import { saturate } from "../../../lib/math/scalar";
import type { GridGeometry } from "../../../lib/field/grid";
import type { WorldRecipe } from "../../recipe";

interface HotspotNode {
  position: readonly [number, number, number];
  strength: number;
  radius: number;
}

export function createHotspotField(recipe: WorldRecipe, grid: GridGeometry): Float32Array {
  const nodes = createNodes(recipe);
  const output = new Float32Array(grid.cellCount);
  for (let index = 0; index < grid.cellCount; index += 1) {
    let value = 0;
    for (const node of nodes) {
      const chordSquared =
        (grid.x[index] - node.position[0]) ** 2 +
        (grid.y[index] - node.position[1]) ** 2 +
        (grid.z[index] - node.position[2]) ** 2;
      value = Math.max(value, node.strength * Math.exp(-chordSquared / node.radius));
    }
    output[index] = saturate(value * recipe.volcanism);
  }
  return output;
}

function createNodes(recipe: WorldRecipe): HotspotNode[] {
  const count = Math.max(2, Math.round(2 + recipe.volcanism * 3 + recipe.spice * 3));
  const nodes: HotspotNode[] = [];
  for (let hotspot = 0; hotspot < count; hotspot += 1) {
    let position = unitVector(recipe.seed, 311, hotspot);
    const axis = unitVector(recipe.seed, 313, hotspot);
    const chainLength = 5 + Math.round(random01(recipe.seed, 317, hotspot) * 5);
    for (let node = 0; node < chainLength; node += 1) {
      nodes.push({
        position,
        strength: (1 - node / (chainLength + 1)) * (0.72 + random01(recipe.seed, 319, hotspot) * 0.28),
        radius: 0.00045 + node * 0.00012
      });
      position = rotate(position, axis, 0.018 + random01(recipe.seed, 331, hotspot) * 0.018);
    }
  }
  return nodes;
}

function unitVector(seed: number, stream: number, index: number): [number, number, number] {
  const azimuth = random01(seed, stream, index) * Math.PI * 2;
  const z = randomSigned(seed, stream + 1, index);
  const radius = Math.sqrt(Math.max(0, 1 - z * z));
  return [Math.cos(azimuth) * radius, Math.sin(azimuth) * radius, z];
}

function rotate(vector: readonly [number, number, number], axis: readonly [number, number, number], angle: number): [number, number, number] {
  const cosine = Math.cos(angle);
  const sine = Math.sin(angle);
  const dot = vector[0] * axis[0] + vector[1] * axis[1] + vector[2] * axis[2];
  return normalize3([
    vector[0] * cosine + (axis[1] * vector[2] - axis[2] * vector[1]) * sine + axis[0] * dot * (1 - cosine),
    vector[1] * cosine + (axis[2] * vector[0] - axis[0] * vector[2]) * sine + axis[1] * dot * (1 - cosine),
    vector[2] * cosine + (axis[0] * vector[1] - axis[1] * vector[0]) * sine + axis[2] * dot * (1 - cosine)
  ]);
}
