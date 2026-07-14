import { createConstraintSet, encodeConstraintSet, type ConstraintSet } from "./constraints";
import type { PackedWorld } from "./model";
import { normalizeRecipe, type WorldRecipe } from "./recipe";
import type { StageProgress } from "./engine/context";
import type { WorldWorkerRequest, WorldWorkerResponse } from "./workerProtocol";

export interface GenerateWorldOptions {
  constraints?: ConstraintSet;
  onProgress?: (progress: StageProgress) => void;
}

export class WorldGenerationCancelledError extends Error {
  constructor() {
    super("World generation was cancelled.");
    this.name = "WorldGenerationCancelledError";
  }
}

export class WorldGeneratorClient {
  private worker: Worker | null = null;
  private requestId = 0;
  private rejectPending: ((reason: Error) => void) | null = null;

  generate(recipeInput: Partial<WorldRecipe>, options: GenerateWorldOptions = {}): Promise<PackedWorld> {
    this.cancel();
    const recipe = normalizeRecipe(recipeInput);
    const constraints = options.constraints ?? createConstraintSet(recipe.width * recipe.height);
    const requestId = ++this.requestId;
    const worker = new Worker(new URL("./world.worker.ts", import.meta.url), { type: "module", name: "bombo-world-engine" });
    this.worker = worker;
    return new Promise((resolve, reject) => {
      this.rejectPending = reject;
      worker.onmessage = (event: MessageEvent<WorldWorkerResponse>) => {
        const message = event.data;
        if (message.requestId !== requestId) return;
        if (message.type === "progress") options.onProgress?.(message.progress);
        if (message.type === "complete") {
          this.disposeWorker(worker);
          resolve(message.world);
        }
        if (message.type === "error") {
          this.disposeWorker(worker);
          reject(new Error(message.message));
        }
      };
      worker.onerror = (event) => {
        this.disposeWorker(worker);
        reject(new Error(event.message || "World worker failed."));
      };
      worker.postMessage({ type: "generate", requestId, recipe, constraints: encodeConstraintSet(constraints) } satisfies WorldWorkerRequest);
    });
  }

  cancel(): void {
    if (!this.worker) return;
    this.worker.terminate();
    this.worker = null;
    const reject = this.rejectPending;
    this.rejectPending = null;
    reject?.(new WorldGenerationCancelledError());
  }

  private disposeWorker(worker: Worker): void {
    worker.terminate();
    if (this.worker === worker) this.worker = null;
    this.rejectPending = null;
  }
}
