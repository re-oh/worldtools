/// <reference lib="webworker" />

import { decodeConstraintSet } from "./constraints";
import { loadSimdFieldMath } from "../runtime/SimdFieldMath";
import { generatePackedWorld } from "./engine/pipeline";
import { worldTransferables } from "./model";
import { normalizeRecipe } from "./recipe";
import type { WorldWorkerRequest, WorldWorkerResponse } from "./workerProtocol";

const worker = self as DedicatedWorkerGlobalScope;
worker.onmessage = async (event: MessageEvent<WorldWorkerRequest>) => {
  const request = event.data;
  try {
    const recipe = normalizeRecipe(request.recipe);
    const constraints = decodeConstraintSet(recipe.width * recipe.height, request.constraints);
    const bulkMath = await loadSimdFieldMath();
    const world = generatePackedWorld(recipe, {
      constraints,
      bulkMath,
      report: (progress) => post({ type: "progress", requestId: request.requestId, progress })
    });
    worker.postMessage({ type: "complete", requestId: request.requestId, world } satisfies WorldWorkerResponse, worldTransferables(world));
  } catch (error) {
    post({
      type: "error",
      requestId: request.requestId,
      message: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined
    });
  }
};

function post(message: WorldWorkerResponse): void {
  worker.postMessage(message);
}
