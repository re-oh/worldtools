import { greatCircleDistanceKm, writeNeighbors } from "../../../lib/field/grid";
import { clamp, saturate } from "../../../lib/math/scalar";
import { hash32, random01 } from "../../../lib/random/deterministic";
import type { NaturalResourceDeposit } from "../../model";
import type { WorldEngineContext } from "../context";
import type { NumericRange, ResourceSystemModel } from "./types";

export function evaluateResourceModels(context: WorldEngineContext, models: readonly ResourceSystemModel[]): NaturalResourceDeposit[] {
  const occurrences: NaturalResourceDeposit[] = [];
  for (const model of models) occurrences.push(...evaluateModel(context, model));
  return occurrences;
}

function evaluateModel(context: WorldEngineContext, model: ResourceSystemModel): NaturalResourceDeposit[] {
  const potential = new Float32Array(context.grid.cellCount);
  const abundance = model.abundance === "organic" ? context.recipe.organicResourceAbundance : context.recipe.mineralAbundance;
  const stream = model.typeId.split("").reduce((value, character) => hash32(value, character.charCodeAt(0)), 0);
  for (let cell = 0; cell < context.grid.cellCount; cell += 1) {
    const spice = 1 + (random01(context.recipe.seed, stream, cell) - 0.5) * context.recipe.spice * 0.18;
    const painted = context.constraints.resourceFavorability[cell] * 0.24;
    potential[cell] = saturate(model.score(context, cell) * Math.sqrt(abundance) * spice + painted);
    for (const output of model.outputs) context.fields[output][cell] = Math.max(context.fields[output][cell], potential[cell]);
  }

  const candidates = localMaxima(potential, context, model.minimumScore);
  const areaScale = clamp(Math.pow(context.recipe.radiusKm / 6371, 2), 0.35, 2.5);
  const target = Math.max(1, Math.round(model.maximumOccurrences * abundance * areaScale));
  const selected: Array<{ cell: number; score: number }> = [];
  for (const candidate of candidates) {
    if (selected.length >= target) break;
    if (selected.every((item) => greatCircleDistanceKm(item.cell, candidate.cell, context.grid) >= model.minimumSpacingKm)) selected.push(candidate);
  }
  const worldAgeMa = context.recipe.worldAgeGa * 1000;
  return selected.map((candidate, sequence) => {
    const randomA = random01(context.recipe.seed, stream + 11, candidate.cell);
    const randomB = random01(context.recipe.seed, stream + 17, candidate.cell);
    const ageMaximum = Math.min(worldAgeMa * 0.98, model.ageMa[1]);
    const ageMinimum = Math.min(ageMaximum, model.ageMa[0]);
    return {
      id: `${model.typeId}-${candidate.cell.toString(36)}`,
      typeId: model.typeId,
      resourceClass: model.resourceClass,
      name: `${model.name} ${sequence + 1}`,
      cell: candidate.cell,
      radiusKm: range(model.radiusKm, randomA),
      ageMa: range([ageMinimum, ageMaximum], randomB),
      potential: candidate.score,
      depthMeters: range(model.depthMeters, randomB),
      thicknessMeters: range(model.thicknessMeters, randomA),
      quality: saturate(candidate.score * 0.74 + randomB * 0.26),
      geometry: model.geometry,
      host: typeof model.host === "function" ? model.host(context, candidate.cell) : model.host,
      setting: model.setting,
      formation: model.formation,
      commodities: model.commodities
    } satisfies NaturalResourceDeposit;
  });
}

function localMaxima(values: Float32Array, context: WorldEngineContext, threshold: number): Array<{ cell: number; score: number }> {
  const result: Array<{ cell: number; score: number }> = [];
  const neighbors = new Int32Array(8);
  for (let cell = 0; cell < values.length; cell += 1) {
    const score = values[cell];
    if (score < threshold) continue;
    const count = writeNeighbors(cell, context.grid, neighbors);
    let maximum = true;
    for (let offset = 0; offset < count; offset += 1) {
      const neighbor = neighbors[offset];
      if (values[neighbor] > score || (values[neighbor] === score && neighbor < cell)) {
        maximum = false;
        break;
      }
    }
    if (maximum) result.push({ cell, score });
  }
  result.sort((a, b) => b.score - a.score || a.cell - b.cell);
  return result;
}

function range([minimum, maximum]: NumericRange, amount: number): number {
  return minimum + (maximum - minimum) * amount;
}
