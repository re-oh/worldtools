import type { FieldId, WorldStageId } from "../model";

export type LayerGroup = "Geology" | "Terrain" | "Ocean" | "Climate" | "Hydrology" | "Cryosphere" | "Soils" | "Ecology" | "Resources";
export type LayerScale = "continuous" | "diverging" | "categorical" | "log";
export type PaletteStop = readonly [number, string];

export interface WorldLayerDefinition {
  id: FieldId;
  label: string;
  group: LayerGroup;
  stage: WorldStageId;
  units: string;
  scale: LayerScale;
  palette: readonly PaletteStop[];
  fixedRange?: readonly [number, number];
  hillshade?: boolean;
  categories?: Readonly<Record<number, string>>;
}

export function defineLayer(
  id: FieldId,
  label: string,
  group: LayerGroup,
  stage: WorldStageId,
  units: string,
  scale: LayerScale,
  palette: readonly PaletteStop[],
  options: Partial<Pick<WorldLayerDefinition, "fixedRange" | "hillshade" | "categories">> = {}
): WorldLayerDefinition {
  return { id, label, group, stage, units, scale, palette, ...options };
}
