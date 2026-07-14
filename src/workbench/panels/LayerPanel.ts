import { layersByGroup, type LayerGroup, type WorldLayerDefinition } from "../../world/layerCatalog";
import type { FieldId } from "../../world/model";

export class LayerPanel {
  private readonly groups = layersByGroup();
  private activeGroup: LayerGroup = "Terrain";
  private activeLayer: FieldId = "elevation";
  private query = "";

  constructor(private readonly container: HTMLElement, private readonly onSelect: (layer: FieldId) => void) {
    this.build();
    container.addEventListener("change", this.onChange);
    container.addEventListener("input", this.onInput);
    container.addEventListener("click", this.onClick);
    this.renderList();
  }

  setActive(layer: FieldId): void {
    this.activeLayer = layer;
    const definition = [...this.groups.values()].flat().find((item) => item.id === layer);
    if (definition) this.activeGroup = definition.group;
    const select = this.container.querySelector<HTMLSelectElement>("[data-layer-group]");
    if (select) select.value = this.activeGroup;
    this.renderList();
  }

  dispose(): void {
    this.container.removeEventListener("change", this.onChange);
    this.container.removeEventListener("input", this.onInput);
    this.container.removeEventListener("click", this.onClick);
  }

  private build(): void {
    const tools = document.createElement("div");
    tools.className = "panel-tools";
    const group = document.createElement("select");
    group.dataset.layerGroup = "";
    group.ariaLabel = "Layer group";
    for (const name of this.groups.keys()) group.add(new Option(name, name));
    group.value = this.activeGroup;
    const search = document.createElement("input");
    search.type = "search";
    search.placeholder = "Filter layers";
    search.dataset.layerSearch = "";
    tools.append(group, search);
    const list = document.createElement("div");
    list.className = "layer-list";
    list.dataset.layerList = "";
    this.container.replaceChildren(tools, list);
  }

  private renderList(): void {
    const list = this.container.querySelector<HTMLElement>("[data-layer-list]");
    if (!list) return;
    const source = this.query
      ? [...this.groups.values()].flat().filter((item) => item.label.toLowerCase().includes(this.query))
      : this.groups.get(this.activeGroup) ?? [];
    list.replaceChildren(...source.map((definition) => layerButton(definition, definition.id === this.activeLayer)));
  }

  private readonly onChange = (event: Event): void => {
    const select = event.target;
    if (!(select instanceof HTMLSelectElement) || select.dataset.layerGroup === undefined) return;
    this.activeGroup = select.value as LayerGroup;
    this.query = "";
    const search = this.container.querySelector<HTMLInputElement>("[data-layer-search]");
    if (search) search.value = "";
    this.renderList();
  };

  private readonly onInput = (event: Event): void => {
    const input = event.target;
    if (!(input instanceof HTMLInputElement) || input.dataset.layerSearch === undefined) return;
    this.query = input.value.trim().toLowerCase();
    this.renderList();
  };

  private readonly onClick = (event: MouseEvent): void => {
    const button = (event.target as Element).closest<HTMLButtonElement>("[data-layer]");
    if (!button?.dataset.layer) return;
    this.onSelect(button.dataset.layer as FieldId);
  };
}

function layerButton(definition: WorldLayerDefinition, active: boolean): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = "layer-button";
  button.dataset.layer = definition.id;
  button.setAttribute("aria-current", String(active));
  const label = document.createElement("span");
  label.textContent = definition.label;
  const units = document.createElement("small");
  units.textContent = definition.units;
  button.append(label, units);
  return button;
}
