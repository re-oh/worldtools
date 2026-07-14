import type { ConstraintSet } from "../../world/constraints";
import type { StageProgress } from "../../world/engine/context";
import type { PackedWorld } from "../../world/model";
import type { WorldRecipe } from "../../world/recipe";
import { WorldGenerationCancelledError, WorldGeneratorClient } from "../../world/WorldGeneratorClient";

export interface GenerationSessionEvents {
  onStart(): void;
  onProgress(progress: StageProgress): void;
  onComplete(world: PackedWorld): void;
  onError(error: unknown): void;
  onFinish(): void;
}

export class GenerationSession {
  private readonly client = new WorldGeneratorClient();
  private serial = 0;

  run(recipe: WorldRecipe, constraints: ConstraintSet, events: GenerationSessionEvents): void {
    const serial = ++this.serial;
    events.onStart();
    void this.client.generate(recipe, {
      constraints,
      onProgress: (progress) => {
        if (serial === this.serial) events.onProgress(progress);
      }
    }).then((world) => {
      if (serial === this.serial) events.onComplete(world);
    }).catch((error: unknown) => {
      if (serial !== this.serial || error instanceof WorldGenerationCancelledError) return;
      events.onError(error);
    }).finally(() => {
      if (serial === this.serial) events.onFinish();
    });
  }

  dispose(): void {
    this.serial += 1;
    this.client.cancel();
  }
}
