import type { ConstraintSet } from "../../world/constraints";
import type { PackedWorld } from "../../world/model";
import { ProjectRepository } from "../../world/ProjectRepository";
import { createProjectArchive, parseProjectArchive, type RestoredProject } from "../../world/projectArchive";

export class ProjectSession {
  private readonly repository = new ProjectRepository();
  private name: string | null = null;
  private expectedChecksum: string | null = null;

  async load(): Promise<RestoredProject | null> {
    const restored = await this.repository.load().catch(() => null);
    if (restored) this.adopt(restored);
    return restored;
  }

  async import(file: File): Promise<RestoredProject> {
    const restored = parseProjectArchive(await file.text());
    this.adopt(restored);
    return restored;
  }

  applyMetadata(world: PackedWorld): PackedWorld {
    if (this.expectedChecksum && world.checksum !== this.expectedChecksum) {
      throw new Error(`Regenerated checksum ${world.checksum} does not match the project checksum ${this.expectedChecksum}.`);
    }
    this.expectedChecksum = null;
    return this.name ? { ...world, name: this.name } : world;
  }

  save(world: PackedWorld, constraints: ConstraintSet): void {
    void this.repository.save(createProjectArchive(world, constraints)).catch(() => undefined);
  }

  export(world: PackedWorld, constraints: ConstraintSet): void {
    const archive = createProjectArchive(world, constraints);
    const url = URL.createObjectURL(new Blob([JSON.stringify(archive)], { type: "application/json" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = `${world.id}.bombo.json`;
    link.click();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
  }

  private adopt(restored: RestoredProject): void {
    this.name = restored.name;
    this.expectedChecksum = restored.expectedChecksum;
  }
}
