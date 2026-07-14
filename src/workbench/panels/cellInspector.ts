import { layerDefinition } from "../../world/layerCatalog";
import { snapshotCell, type FieldId, type PackedWorld } from "../../world/model";
import { emptyPanel, metricSection } from "./panelDom";

export function renderCellInspector(container: HTMLElement, world: PackedWorld | null, cell: number | null, activeLayer: FieldId): void {
  if (!world || cell === null) {
    container.replaceChildren(emptyPanel("No cell selected"));
    return;
  }
  const snapshot = snapshotCell(world, cell);
  const content = document.createElement("div");
  content.className = "inspector-content";
  const heading = document.createElement("h2");
  heading.className = "inspector-heading";
  heading.textContent = `Cell ${snapshot.index.toLocaleString()}`;
  const subheading = document.createElement("p");
  subheading.className = "inspector-subheading";
  subheading.textContent = `${coordinate(snapshot.latitudeDegrees, "N", "S")}, ${coordinate(snapshot.longitudeDegrees, "E", "W")}`;
  content.append(heading, subheading);

  const active = layerDefinition(activeLayer);
  content.append(metricSection(active.label, [[active.units, formatField(world, activeLayer, snapshot.values[activeLayer])]]));
  content.append(metricSection("Crust and relief", [
    ["Plate", formatField(world, "plateId", snapshot.values.plateId)],
    ["Lithology", formatField(world, "lithology", snapshot.values.lithology)],
    ["Crust age", formatField(world, "crustAge", snapshot.values.crustAge)],
    ["Elevation", formatField(world, "elevation", snapshot.values.elevation)],
    ["Uplift", formatField(world, "uplift", snapshot.values.uplift)],
    ["Basin", formatField(world, "sedimentaryBasin", snapshot.values.sedimentaryBasin)]
  ]));
  content.append(metricSection("Water and climate", [
    ["Temperature", formatField(world, "temperature", snapshot.values.temperature)],
    ["Seasonality", formatField(world, "temperatureSeasonality", snapshot.values.temperatureSeasonality)],
    ["Precipitation", formatField(world, "precipitation", snapshot.values.precipitation)],
    ["River flow", formatField(world, "flow", snapshot.values.flow)],
    ["Floodplain", formatField(world, "floodplain", snapshot.values.floodplain)],
    ["Persistent ice", formatField(world, "ice", snapshot.values.ice)]
  ]));
  content.append(metricSection("Soil and cover", [
    ["Soil", formatField(world, "soilType", snapshot.values.soilType)],
    ["Clay", formatField(world, "clayFraction", snapshot.values.clayFraction)],
    ["Organic carbon", formatPercent(snapshot.values.organicCarbon)],
    ["pH", snapshot.values.soilPH > 0 ? snapshot.values.soilPH.toFixed(1) : "None"],
    ["Biome", formatField(world, "biome", snapshot.values.biome)],
    ["Forest", formatPercent(snapshot.values.forestCover)],
    ["Grassland", formatPercent(snapshot.values.grassCover)],
    ["Wetland", formatPercent(snapshot.values.wetlandCover)]
  ]));
  container.replaceChildren(content);
}

export function formatField(world: PackedWorld, id: FieldId, value: number): string {
  const definition = layerDefinition(id);
  if (definition.categories) return definition.categories[Math.round(value)] ?? `Class ${Math.round(value)}`;
  if (definition.units === "id" || definition.units === "class") return String(Math.round(value));
  if (definition.units === "fraction" || definition.units === "potential" || definition.units === "index" || definition.units === "bias") return formatPercent(value);
  if (definition.units === "C") return `${value.toFixed(1)} °C`;
  if (definition.units === "m") return `${Math.round(value).toLocaleString()} m`;
  if (definition.units === "mm/yr") return `${Math.round(value).toLocaleString()} mm/yr`;
  if (definition.units === "Ma") return `${Math.round(value).toLocaleString()} Ma`;
  if (definition.units === "km") return `${value.toFixed(1)} km`;
  return Math.abs(value) >= 100 ? Math.round(value).toLocaleString() : value.toFixed(3);
}

function coordinate(value: number, positive: string, negative: string): string {
  return `${Math.abs(value).toFixed(2)}°${value >= 0 ? positive : negative}`;
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}
