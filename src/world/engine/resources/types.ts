import type { FieldId, NaturalResourceDeposit } from "../../model";
import type { WorldEngineContext } from "../context";

export type NumericRange = readonly [number, number];

export interface ResourceSystemModel {
  typeId: NaturalResourceDeposit["typeId"];
  name: string;
  resourceClass: NaturalResourceDeposit["resourceClass"];
  abundance: "mineral" | "organic";
  outputs: readonly FieldId[];
  geometry: NaturalResourceDeposit["geometry"];
  host: string | ((context: WorldEngineContext, cell: number) => string);
  setting: string;
  formation: string;
  commodities: readonly string[];
  minimumScore: number;
  maximumOccurrences: number;
  minimumSpacingKm: number;
  radiusKm: NumericRange;
  ageMa: NumericRange;
  depthMeters: NumericRange;
  thicknessMeters: NumericRange;
  score(context: WorldEngineContext, cell: number): number;
}
