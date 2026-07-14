import { del, get, set } from "idb-keyval";
import { restoreProjectArchive, type WorldProjectArchive, type RestoredProject } from "./projectArchive";
import { WORLD_VERSION } from "./recipe";

const ACTIVE_PROJECT_KEY = `bombo.active-project.v${WORLD_VERSION}`;

export class ProjectRepository {
  async save(archive: WorldProjectArchive): Promise<void> {
    await set(ACTIVE_PROJECT_KEY, archive);
  }

  async load(): Promise<RestoredProject | null> {
    const stored = await get<unknown>(ACTIVE_PROJECT_KEY);
    return stored ? restoreProjectArchive(stored) : null;
  }

  async clear(): Promise<void> {
    await del(ACTIVE_PROJECT_KEY);
  }
}
