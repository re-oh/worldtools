import { ENVIRONMENT_LAYERS } from "./layers/environment";
import { GEOLOGY_LAYERS } from "./layers/geology";
import { RESOURCE_LAYERS } from "./layers/resources";
import type { LayerGroup, LayerScale, PaletteStop, WorldLayerDefinition } from "./layers/types";
import { FIELD_IDS, type FieldId } from "./model";

export type { LayerGroup, LayerScale, PaletteStop, WorldLayerDefinition } from "./layers/types";

export const WORLD_LAYERS: readonly WorldLayerDefinition[] = [...GEOLOGY_LAYERS, ...ENVIRONMENT_LAYERS, ...RESOURCE_LAYERS];
export const DEFAULT_LAYER_ID: FieldId = "elevation";

const byId = new Map(WORLD_LAYERS.map((definition) => [definition.id, definition]));
for (const id of FIELD_IDS) {
  if (!byId.has(id)) throw new Error(`World field ${id} has no layer definition.`);
}

export function layerDefinition(id: FieldId): WorldLayerDefinition {
  const definition = byId.get(id);
  if (!definition) throw new Error(`Unknown world layer: ${id}`);
  return definition;
}

export function layersByGroup(): ReadonlyMap<LayerGroup, readonly WorldLayerDefinition[]> {
  const groups = new Map<LayerGroup, WorldLayerDefinition[]>();
  for (const definition of WORLD_LAYERS) {
    const values = groups.get(definition.group) ?? [];
    values.push(definition);
    groups.set(definition.group, values);
  }
  return groups;
}
