import { createGridGeometry, type GridGeometry } from "../lib/field/grid";
import {
  createConstraintSet,
  resampleConstraintSet,
  type ConstraintSet
} from "../world/constraints";
import { type FieldId, type PackedWorld, type WorldStageId } from "../world/model";
import { DEFAULT_RECIPE, normalizeRecipe, recipeForPreset, type ResolutionPreset, type WorldRecipe } from "../world/recipe";
import { GenerationSession } from "./controllers/GenerationSession";
import { GenerationMonitor } from "./controllers/GenerationMonitor";
import { ProjectSession } from "./controllers/ProjectSession";
import { ConstraintEditor } from "./editing/ConstraintEditor";
import { WorkbenchEvents } from "./events/WorkbenchEvents";
import { LayerPanel } from "./panels/LayerPanel";
import { ParameterPanel } from "./panels/ParameterPanel";
import { renderCellInspector } from "./panels/cellInspector";
import { renderResourceInspector } from "./panels/resourceInspector";
import { PARAMETER_DEFINITIONS_BY_KEY, type EditableRecipeKey } from "./parameters/definitions";
import { createWorkbenchShell, type WorkbenchShell } from "./shell/createShell";
import { WorldViewport } from "./viewport/WorldViewport";

export class WorldWorkbench {
  private readonly shell: WorkbenchShell;
  private readonly generation = new GenerationSession();
  private readonly monitor: GenerationMonitor;
  private readonly projects = new ProjectSession();
  private readonly editor: ConstraintEditor;
  private readonly viewport: WorldViewport;
  private readonly layerPanel: LayerPanel;
  private readonly parameterPanel: ParameterPanel;
  private readonly events: WorkbenchEvents;
  private recipe: WorldRecipe = recipeForPreset(DEFAULT_RECIPE, "draft");
  private constraints: ConstraintSet = createConstraintSet(this.recipe.width * this.recipe.height);
  private world: PackedWorld | null = null;
  private grid: GridGeometry | null = null;
  private activeLayer: FieldId = "elevation";
  private selectedCell: number | null = null;
  private running = false;
  private dirty = true;
  private autoGenerateTimer: number | null = null;

  constructor(root: HTMLElement) {
    this.shell = createWorkbenchShell(root);
    this.monitor = new GenerationMonitor(this.shell);
    this.editor = new ConstraintEditor(this.shell, (stage) => this.constraintsChanged(stage));
    this.editor.setWorld(this.recipe, this.constraints);
    this.viewport = new WorldViewport(this.shell, this.editor.painter, {
      onSelect: (cell) => this.selectCell(cell),
      onPaintEnd: () => this.editor.refreshHistoryControls()
    });
    this.layerPanel = new LayerPanel(this.shell.layerPanel, (layer) => this.setLayer(layer));
    this.parameterPanel = new ParameterPanel(this.shell.parameterPanel, {
      onChange: (key, value) => this.updateParameter(key, value),
      onPreset: (preset) => this.updateResolution(preset),
      onGenerate: () => this.generate()
    });
    this.events = new WorkbenchEvents(this.shell.root, {
      onCommand: (command) => this.handleCommand(command),
      onMode: (mode) => this.viewport.setMode(mode),
      onPaintSign: (sign) => this.editor.setSign(sign),
      onLeftTab: (tab) => this.showLeftTab(tab),
      onInspectorTab: (tab) => this.showInspectorTab(tab)
    });
    this.shell.projectInput.addEventListener("change", this.onProjectFile);
    this.renderPanels();
  }

  async start(): Promise<void> {
    const restored = await this.projects.load();
    if (restored) {
      this.recipe = restored.recipe;
      this.constraints = restored.constraints;
      this.editor.setWorld(this.recipe, this.constraints);
    }
    await this.viewport.start();
    this.generate();
  }

  dispose(): void {
    if (this.autoGenerateTimer !== null) clearTimeout(this.autoGenerateTimer);
    this.generation.dispose();
    this.viewport.dispose();
    this.editor.dispose();
    this.layerPanel.dispose();
    this.parameterPanel.dispose();
    this.events.dispose();
    this.shell.projectInput.removeEventListener("change", this.onProjectFile);
  }

  private generate(): void {
    this.generation.run(this.recipe, this.constraints, {
      onStart: () => {
        this.running = true;
        this.viewport.setMode("inspect");
        this.monitor.begin();
        this.renderPanels();
      },
      onProgress: (progress) => this.monitor.update(progress),
      onComplete: (world) => this.installWorld(this.projects.applyMetadata(world)),
      onError: (error) => this.monitor.fail(error),
      onFinish: () => {
        this.running = false;
        this.monitor.finish();
        this.renderPanels();
      }
    });
  }

  private installWorld(world: PackedWorld): void {
    this.world = world;
    this.recipe = world.recipe;
    this.grid = createGridGeometry(world.recipe.width, world.recipe.height, world.recipe.radiusKm);
    this.dirty = false;
    this.monitor.complete(world.stages);
    this.viewport.setWorld(world);
    this.viewport.setLayer(this.activeLayer);
    this.selectedCell = pickInitialCell(world);
    this.viewport.setSelection(this.selectedCell);
    this.editor.setWorld(this.recipe, this.constraints);
    this.shell.worldName.textContent = world.name;
    this.shell.checksum.textContent = `${world.checksum} | ${world.generationMs.toFixed(0)} ms`;
    this.renderPanels();
    this.renderInspectors();
    this.projects.save(world, this.constraints);
  }

  private updateParameter(key: EditableRecipeKey, value: number): void {
    this.recipe = normalizeRecipe({ ...this.recipe, [key]: value });
    this.editor.setWorld(this.recipe, this.constraints);
    this.dirty = true;
    this.monitor.invalidate(PARAMETER_DEFINITIONS_BY_KEY[key].invalidates);
    this.renderPanels();
  }

  private updateResolution(preset: ResolutionPreset): void {
    const previous = this.recipe;
    const next = recipeForPreset(this.recipe, preset);
    this.constraints = resampleConstraintSet(this.constraints, previous.width, previous.height, next.width, next.height);
    this.recipe = next;
    this.editor.clearHistory();
    this.editor.setWorld(this.recipe, this.constraints);
    this.dirty = true;
    this.monitor.invalidate("geology");
    this.renderPanels();
    this.generate();
  }

  private setLayer(layer: FieldId): void {
    this.activeLayer = layer;
    this.viewport.setLayer(layer);
    this.layerPanel.setActive(layer);
    this.renderInspectors();
  }

  private selectCell(cell: number): void {
    this.selectedCell = cell;
    this.viewport.setSelection(cell);
    this.renderInspectors();
  }

  private constraintsChanged(stage: WorldStageId): void {
    this.dirty = true;
    this.monitor.invalidate(stage);
    this.editor.refreshHistoryControls();
    this.renderPanels();
    if (this.autoGenerateTimer !== null) clearTimeout(this.autoGenerateTimer);
    this.autoGenerateTimer = window.setTimeout(() => {
      this.autoGenerateTimer = null;
      this.generate();
    }, 420);
  }

  private renderPanels(): void {
    this.parameterPanel.render(this.recipe, this.dirty, this.running);
    const generate = this.shell.root.querySelector<HTMLButtonElement>("[data-action=generate]");
    if (generate) generate.disabled = this.running;
    this.monitor.renderPipeline();
  }

  private renderInspectors(): void {
    renderCellInspector(this.shell.cellInspector, this.world, this.selectedCell, this.activeLayer);
    renderResourceInspector(this.shell.resourceInspector, this.world, this.selectedCell, this.grid);
  }

  private handleCommand(command: string): void {
    switch (command) {
      case "generate": this.generate(); break;
      case "randomize": this.randomize(); break;
      case "undo": this.editor.undo(); break;
      case "redo": this.editor.redo(); break;
      case "reset-view": this.viewport.resetView(); break;
      case "zoom-in": this.viewport.zoomBy(-260); break;
      case "zoom-out": this.viewport.zoomBy(260); break;
      case "toggle-layers": this.shell.leftPanel.classList.toggle("open"); break;
      case "toggle-inspector": this.shell.rightPanel.classList.toggle("open"); break;
      case "save-project": this.saveProject(); break;
      case "export-project": this.exportProject(); break;
      case "import-project": this.shell.projectInput.click(); break;
    }
  }

  private showLeftTab(tab: string): void {
    this.shell.layerPanel.hidden = tab !== "layers";
    this.shell.parameterPanel.hidden = tab !== "parameters";
    for (const button of this.shell.root.querySelectorAll<HTMLButtonElement>("[data-left-tab]")) button.setAttribute("aria-selected", String(button.dataset.leftTab === tab));
  }

  private showInspectorTab(tab: string): void {
    this.shell.cellInspector.hidden = tab !== "cell";
    this.shell.resourceInspector.hidden = tab !== "resources";
    this.shell.pipelineInspector.hidden = tab !== "pipeline";
    for (const button of this.shell.root.querySelectorAll<HTMLButtonElement>("[data-inspector-tab]")) button.setAttribute("aria-selected", String(button.dataset.inspectorTab === tab));
  }

  private randomize(): void {
    const random = new Uint32Array(1);
    crypto.getRandomValues(random);
    this.recipe = normalizeRecipe({ ...this.recipe, seed: random[0] });
    this.dirty = true;
    this.monitor.invalidate("geology");
    this.renderPanels();
    this.generate();
  }

  private saveProject(): void {
    if (!this.world) return;
    this.projects.save(this.world, this.constraints);
  }

  private exportProject(): void {
    if (!this.world) return;
    this.projects.export(this.world, this.constraints);
  }

  private readonly onProjectFile = async (): Promise<void> => {
    const file = this.shell.projectInput.files?.[0];
    this.shell.projectInput.value = "";
    if (!file) return;
    try {
      const restored = await this.projects.import(file);
      this.recipe = restored.recipe;
      this.constraints = restored.constraints;
      this.editor.clearHistory();
      this.editor.setWorld(this.recipe, this.constraints);
      this.dirty = true;
      this.monitor.invalidate("geology");
      this.generate();
    } catch (error) {
      this.monitor.fail(error);
    }
  };
}

function pickInitialCell(world: PackedWorld): number {
  let best = 0;
  let bestScore = Number.NEGATIVE_INFINITY;
  for (let cell = 0; cell < world.recipe.width * world.recipe.height; cell += 1) {
    if (world.fields.waterDepth[cell] > 0) continue;
    const score = world.fields.uplift[cell] * 0.45 + world.fields.flow[cell] * 0.38 + world.fields.delta[cell] * 0.22 + world.fields.forestCover[cell] * 0.12;
    if (score > bestScore) {
      bestScore = score;
      best = cell;
    }
  }
  return best;
}
