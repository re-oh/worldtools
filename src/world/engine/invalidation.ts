import { STAGE_IDS, type WorldStageId, type WorldStageStatus } from "../model";

export function invalidateStages(
  stages: readonly WorldStageStatus[],
  from: WorldStageId,
  detail = "Inputs changed; regeneration required"
): WorldStageStatus[] {
  const firstStale = STAGE_IDS.indexOf(from);
  return stages.map((stage, index) => index < firstStale
    ? { ...stage }
    : { ...stage, state: "stale", progress: 0, detail });
}
