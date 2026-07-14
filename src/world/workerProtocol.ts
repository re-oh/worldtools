import type { SparseConstraintLayer } from "./constraints";
import type { PackedWorld } from "./model";
import type { WorldRecipe } from "./recipe";
import type { StageProgress } from "./engine/context";

export interface GenerateWorldRequest {
  type: "generate";
  requestId: number;
  recipe: Partial<WorldRecipe>;
  constraints: SparseConstraintLayer[];
}

export type WorldWorkerRequest = GenerateWorldRequest;

export type WorldWorkerResponse =
  | { type: "progress"; requestId: number; progress: StageProgress }
  | { type: "complete"; requestId: number; world: PackedWorld }
  | { type: "error"; requestId: number; message: string; stack?: string };
