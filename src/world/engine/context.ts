import type { GridGeometry } from "../../lib/field/grid";
import { SCALAR_BULK_FIELD_MATH, type BulkFieldMath } from "../../lib/field/BulkFieldMath";
import type { ConstraintSet } from "../constraints";
import { createEmptyFieldMap, type FieldMap, type NaturalResourceDeposit, type PlateModel, type WorldStageId } from "../model";
import type { WorldRecipe } from "../recipe";

export interface WorldEngineContext {
  recipe: WorldRecipe;
  constraints: ConstraintSet;
  grid: GridGeometry;
  fields: FieldMap;
  plates: PlateModel[];
  deposits: NaturalResourceDeposit[];
  downstream: Int32Array;
  basin: Int32Array;
  scratch: EngineScratch;
  bulkMath: BulkFieldMath;
  notes: string[];
}

export interface EngineScratch {
  currentX: Float32Array;
  currentY: Float32Array;
  windX: Float32Array;
  windY: Float32Array;
  filledElevation: Float32Array;
  streamOrder: Uint8Array;
}

export interface StageProgress {
  stageId: WorldStageId;
  label: string;
  progress: number;
  detail: string;
}

export type ProgressReporter = (progress: StageProgress) => void;

export function createEngineContext(recipe: WorldRecipe, constraints: ConstraintSet, grid: GridGeometry, bulkMath: BulkFieldMath = SCALAR_BULK_FIELD_MATH): WorldEngineContext {
  return {
    recipe,
    constraints,
    grid,
    fields: createEmptyFieldMap(grid.cellCount),
    plates: [],
    deposits: [],
    downstream: new Int32Array(grid.cellCount).fill(-1),
    basin: new Int32Array(grid.cellCount).fill(-1),
    scratch: {
      currentX: new Float32Array(grid.cellCount),
      currentY: new Float32Array(grid.cellCount),
      windX: new Float32Array(grid.cellCount),
      windY: new Float32Array(grid.cellCount),
      filledElevation: new Float32Array(grid.cellCount),
      streamOrder: new Uint8Array(grid.cellCount)
    },
    bulkMath,
    notes: [`Bulk field math: ${bulkMath.backend}`]
  };
}
