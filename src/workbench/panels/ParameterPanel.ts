import { clamp } from "../../lib/math/scalar";
import { RESOLUTION_PRESETS, type ResolutionPreset, type WorldRecipe } from "../../world/recipe";
import { PARAMETER_DEFINITIONS, PARAMETER_GROUPS, type EditableRecipeKey, type ParameterDefinition } from "../parameters/definitions";

export interface ParameterPanelEvents {
  onChange(key: EditableRecipeKey, value: number): void;
  onPreset(preset: ResolutionPreset): void;
  onGenerate(): void;
}

export class ParameterPanel {
  private recipe: WorldRecipe | null = null;
  private dirty = true;
  private running = false;

  constructor(private readonly container: HTMLElement, private readonly events: ParameterPanelEvents) {
    this.build();
    container.addEventListener("input", this.onInput);
    container.addEventListener("change", this.onChange);
    container.addEventListener("click", this.onClick);
  }

  render(recipe: WorldRecipe, dirty: boolean, running: boolean): void {
    this.recipe = recipe;
    this.dirty = dirty;
    this.running = running;
    for (const definition of PARAMETER_DEFINITIONS) {
      const inputs = this.container.querySelectorAll<HTMLInputElement>(`[data-param="${definition.key}"]`);
      for (const input of inputs) input.value = String(recipe[definition.key]);
      const output = this.container.querySelector<HTMLOutputElement>(`output[data-value="${definition.key}"]`);
      if (output) output.value = formatValue(definition, recipe[definition.key]);
    }
    const preset = currentPreset(recipe);
    for (const button of this.container.querySelectorAll<HTMLButtonElement>("[data-preset]")) button.setAttribute("aria-pressed", String(button.dataset.preset === preset));
    const generate = this.container.querySelector<HTMLButtonElement>("[data-panel-action=generate]");
    if (generate) {
      generate.disabled = running;
      generate.textContent = running ? "Generating" : dirty ? "Apply parameters" : "Regenerate";
    }
  }

  dispose(): void {
    this.container.removeEventListener("input", this.onInput);
    this.container.removeEventListener("change", this.onChange);
    this.container.removeEventListener("click", this.onClick);
  }

  private build(): void {
    const resolution = document.createElement("section");
    resolution.className = "resolution-control";
    resolution.innerHTML = `<div class="segmented">${Object.keys(RESOLUTION_PRESETS).map((preset) => `<button data-preset="${preset}" aria-pressed="false">${capitalize(preset)}</button>`).join("")}</div>`;
    const sections = document.createElement("div");
    sections.className = "parameter-sections";
    for (const group of PARAMETER_GROUPS) {
      const details = document.createElement("details");
      details.open = group === "World" || group === "Climate";
      const summary = document.createElement("summary");
      summary.textContent = group;
      details.append(summary);
      for (const definition of PARAMETER_DEFINITIONS.filter((item) => item.group === group)) details.append(createControl(definition));
      sections.append(details);
    }
    const actions = document.createElement("div");
    actions.className = "parameter-actions";
    actions.innerHTML = `<button class="command-button" data-panel-action="generate">Apply parameters</button>`;
    this.container.replaceChildren(resolution, sections, actions);
  }

  private readonly onInput = (event: Event): void => {
    const input = event.target;
    if (!(input instanceof HTMLInputElement) || !input.dataset.param) return;
    const definition = definitionFor(input.dataset.param);
    const value = clamp(Number(input.value), definition.minimum, definition.maximum);
    const output = this.container.querySelector<HTMLOutputElement>(`output[data-value="${definition.key}"]`);
    if (output) output.value = formatValue(definition, value);
    const peerSelector = input.type === "range" ? `input[type=number][data-param="${definition.key}"]` : `input[type=range][data-param="${definition.key}"]`;
    const peer = this.container.querySelector<HTMLInputElement>(peerSelector);
    if (peer) peer.value = String(value);
    if (input.type === "range") this.events.onChange(definition.key, value);
  };

  private readonly onChange = (event: Event): void => {
    const input = event.target;
    if (!(input instanceof HTMLInputElement) || !input.dataset.param) return;
    const definition = definitionFor(input.dataset.param);
    this.events.onChange(definition.key, clamp(Number(input.value), definition.minimum, definition.maximum));
  };

  private readonly onClick = (event: MouseEvent): void => {
    const button = (event.target as Element).closest<HTMLButtonElement>("button");
    if (!button) return;
    if (button.dataset.panelAction === "generate") this.events.onGenerate();
    else if (button.dataset.preset) this.events.onPreset(button.dataset.preset as ResolutionPreset);
  };
}

function createControl(definition: ParameterDefinition): HTMLElement {
  const row = document.createElement("div");
  row.className = "parameter-control";
  const label = document.createElement("label");
  label.htmlFor = `parameter-${definition.key}`;
  label.innerHTML = `<span>${definition.label}</span><output data-value="${definition.key}"></output>`;
  row.append(label);
  if (definition.control !== "number") {
    const slider = inputFor(definition, "range");
    slider.id = `parameter-${definition.key}`;
    row.append(slider);
  }
  const number = inputFor(definition, "number");
  if (definition.control === "number") number.id = `parameter-${definition.key}`;
  row.append(number);
  return row;
}

function inputFor(definition: ParameterDefinition, type: "range" | "number"): HTMLInputElement {
  const input = document.createElement("input");
  input.type = type;
  input.min = String(definition.minimum);
  input.max = String(definition.maximum);
  input.step = String(definition.step);
  input.dataset.param = definition.key;
  return input;
}

function definitionFor(key: string): ParameterDefinition {
  const definition = PARAMETER_DEFINITIONS.find((item) => item.key === key);
  if (!definition) throw new Error(`Unknown parameter ${key}.`);
  return definition;
}

function formatValue(definition: ParameterDefinition, value: number): string {
  return definition.format?.(value) ?? String(value);
}

function currentPreset(recipe: WorldRecipe): ResolutionPreset | null {
  return (Object.entries(RESOLUTION_PRESETS).find(([, resolution]) => resolution.width === recipe.width && resolution.height === recipe.height)?.[0] as ResolutionPreset | undefined) ?? null;
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
