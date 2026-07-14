import {
  Cpu,
  createIcons,
  FileDown,
  FolderUp,
  Hand,
  Layers3,
  Maximize2,
  MousePointer2,
  Paintbrush,
  PanelRight,
  Play,
  Redo2,
  Save,
  Shuffle,
  SlidersHorizontal,
  Undo2,
  ZoomIn,
  ZoomOut
} from "lucide";

export interface WorkbenchShell {
  root: HTMLElement;
  canvas: HTMLCanvasElement;
  viewport: HTMLElement;
  leftPanel: HTMLElement;
  rightPanel: HTMLElement;
  layerPanel: HTMLElement;
  parameterPanel: HTMLElement;
  cellInspector: HTMLElement;
  resourceInspector: HTMLElement;
  pipelineInspector: HTMLElement;
  activeLayerTitle: HTMLElement;
  worldName: HTMLElement;
  backend: HTMLElement;
  coordinates: HTMLElement;
  checksum: HTMLElement;
  generation: HTMLElement;
  generationStage: HTMLElement;
  progress: HTMLElement;
  paintBar: HTMLElement;
  constraintSelect: HTMLSelectElement;
  brushRadius: HTMLInputElement;
  brushStrength: HTMLInputElement;
  brushCursor: HTMLElement;
  projectInput: HTMLInputElement;
}

export function createWorkbenchShell(root: HTMLElement): WorkbenchShell {
  root.innerHTML = `
    <div class="workbench">
      <header class="topbar">
        <div class="brand"><strong>WORLDTOOLS</strong><span id="world-name">New deterministic world</span></div>
        <div class="active-layer-title" id="active-layer-title">Relief</div>
        <div class="top-actions">
          <button class="icon-button mobile-only" data-action="toggle-layers" title="Layers and parameters" aria-label="Layers and parameters"><i data-lucide="layers-3"></i></button>
          <button class="icon-button" data-action="save-project" title="Save project" aria-label="Save project"><i data-lucide="save"></i></button>
          <button class="icon-button" data-action="export-project" title="Export project" aria-label="Export project"><i data-lucide="file-down"></i></button>
          <button class="icon-button" data-action="import-project" title="Import project" aria-label="Import project"><i data-lucide="folder-up"></i></button>
          <button class="icon-button" data-action="randomize" title="Randomize seed" aria-label="Randomize seed"><i data-lucide="shuffle"></i></button>
          <button class="command-button" data-action="generate"><i data-lucide="play"></i><span>Generate</span></button>
          <button class="icon-button mobile-only" data-action="toggle-inspector" title="Inspector" aria-label="Inspector"><i data-lucide="panel-right"></i></button>
        </div>
        <div class="generation-line"><span id="generation-progress"></span></div>
      </header>
      <main class="workspace">
        <aside class="side-panel left-panel" id="left-panel">
          <div class="panel-tabs">
            <button data-left-tab="layers" aria-selected="true">Layers</button>
            <button data-left-tab="parameters" aria-selected="false">Parameters</button>
          </div>
          <div class="panel-body" id="layer-panel"></div>
          <div class="panel-body" id="parameter-panel" hidden></div>
        </aside>
        <section class="viewport" id="viewport" data-mode="inspect" data-dragging="false">
          <canvas id="world-canvas"></canvas>
          <div class="map-toolbar" role="toolbar" aria-label="Map tools">
            <button class="mode-button" data-mode="inspect" aria-pressed="true" title="Inspect" aria-label="Inspect"><i data-lucide="mouse-pointer-2"></i></button>
            <button class="mode-button" data-mode="pan" aria-pressed="false" title="Pan" aria-label="Pan"><i data-lucide="hand"></i></button>
            <button class="mode-button" data-mode="paint" aria-pressed="false" title="Paint constraints" aria-label="Paint constraints"><i data-lucide="paintbrush"></i></button>
            <span class="divider"></span>
            <button class="icon-button" data-action="undo" title="Undo" aria-label="Undo" disabled><i data-lucide="undo-2"></i></button>
            <button class="icon-button" data-action="redo" title="Redo" aria-label="Redo" disabled><i data-lucide="redo-2"></i></button>
            <span class="divider"></span>
            <button class="icon-button" data-action="zoom-out" title="Zoom out" aria-label="Zoom out"><i data-lucide="zoom-out"></i></button>
            <button class="icon-button" data-action="reset-view" title="Fit world" aria-label="Fit world"><i data-lucide="maximize-2"></i></button>
            <button class="icon-button" data-action="zoom-in" title="Zoom in" aria-label="Zoom in"><i data-lucide="zoom-in"></i></button>
          </div>
          <div class="paint-bar" id="paint-bar" hidden>
            <select id="constraint-select" aria-label="Constraint layer"></select>
            <div class="segmented">
              <button data-paint-sign="1" aria-pressed="true">Add</button>
              <button data-paint-sign="-1" aria-pressed="false">Subtract</button>
              <button data-paint-sign="0" aria-pressed="false">Erase</button>
            </div>
            <label>Radius <input id="brush-radius" type="range" min="50" max="1400" step="25" value="350"></label>
            <label>Strength <input id="brush-strength" type="range" min="0.1" max="2" step="0.1" value="1"></label>
          </div>
          <div class="generation-state" id="generation-state" hidden><span class="spinner"></span><strong id="generation-stage">Preparing world</strong></div>
          <div class="brush-cursor" id="brush-cursor" hidden></div>
        </section>
        <aside class="side-panel right-panel" id="right-panel">
          <div class="panel-tabs three">
            <button data-inspector-tab="cell" aria-selected="true">Cell</button>
            <button data-inspector-tab="resources" aria-selected="false">Resources</button>
            <button data-inspector-tab="pipeline" aria-selected="false">Pipeline</button>
          </div>
          <div class="panel-body" id="cell-inspector"></div>
          <div class="panel-body" id="resource-inspector" hidden></div>
          <div class="panel-body" id="pipeline-inspector" hidden></div>
        </aside>
      </main>
      <footer class="statusbar">
        <span class="backend" id="backend"><i data-lucide="cpu"></i> Renderer pending</span>
        <span id="checksum" class="checksum">No checksum</span>
        <span id="coordinates" class="coordinates">No cell selected</span>
      </footer>
      <input id="project-input" type="file" accept=".json,.bombo" hidden>
    </div>`;
  createIcons({ icons: { Cpu, FileDown, FolderUp, Hand, Layers3, Maximize2, MousePointer2, Paintbrush, PanelRight, Play, Redo2, Save, Shuffle, SlidersHorizontal, Undo2, ZoomIn, ZoomOut } });
  return {
    root,
    canvas: required(root, "#world-canvas", HTMLCanvasElement),
    viewport: required(root, "#viewport", HTMLElement),
    leftPanel: required(root, "#left-panel", HTMLElement),
    rightPanel: required(root, "#right-panel", HTMLElement),
    layerPanel: required(root, "#layer-panel", HTMLElement),
    parameterPanel: required(root, "#parameter-panel", HTMLElement),
    cellInspector: required(root, "#cell-inspector", HTMLElement),
    resourceInspector: required(root, "#resource-inspector", HTMLElement),
    pipelineInspector: required(root, "#pipeline-inspector", HTMLElement),
    activeLayerTitle: required(root, "#active-layer-title", HTMLElement),
    worldName: required(root, "#world-name", HTMLElement),
    backend: required(root, "#backend", HTMLElement),
    coordinates: required(root, "#coordinates", HTMLElement),
    checksum: required(root, "#checksum", HTMLElement),
    generation: required(root, "#generation-state", HTMLElement),
    generationStage: required(root, "#generation-stage", HTMLElement),
    progress: required(root, "#generation-progress", HTMLElement),
    paintBar: required(root, "#paint-bar", HTMLElement),
    constraintSelect: required(root, "#constraint-select", HTMLSelectElement),
    brushRadius: required(root, "#brush-radius", HTMLInputElement),
    brushStrength: required(root, "#brush-strength", HTMLInputElement),
    brushCursor: required(root, "#brush-cursor", HTMLElement),
    projectInput: required(root, "#project-input", HTMLInputElement)
  };
}

function required<T extends Element>(root: ParentNode, selector: string, type: { new(): T }): T {
  const element = root.querySelector(selector);
  if (!(element instanceof type)) throw new Error(`Workbench element ${selector} is missing.`);
  return element;
}
