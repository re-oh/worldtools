import { greatCircleDistanceKm, type GridGeometry } from "../../lib/field/grid";
import type { NaturalResourceDeposit, PackedWorld } from "../../world/model";
import { emptyPanel } from "./panelDom";

interface NearbyResource {
  deposit: NaturalResourceDeposit;
  distanceKm: number;
}

export function renderResourceInspector(container: HTMLElement, world: PackedWorld | null, cell: number | null, grid: GridGeometry | null): void {
  if (!world || cell === null || !grid) {
    container.replaceChildren(emptyPanel("No resource context"));
    return;
  }
  const resources: NearbyResource[] = world.deposits
    .map((deposit) => ({ deposit, distanceKm: greatCircleDistanceKm(cell, deposit.cell, grid) }))
    .sort((a, b) => a.distanceKm - b.distanceKm || b.deposit.potential - a.deposit.potential);
  const local = resources.filter(({ deposit, distanceKm }) => distanceKm <= Math.max(350, deposit.radiusKm * 2.5));
  const shown = (local.length > 0 ? local : resources).slice(0, local.length > 0 ? 18 : 6);
  if (shown.length === 0) {
    container.replaceChildren(emptyPanel("No resource occurrences in this world"));
    return;
  }
  const fragment = document.createDocumentFragment();
  const header = document.createElement("div");
  header.className = "inspector-content";
  const heading = document.createElement("h2");
  heading.className = "inspector-heading";
  heading.textContent = local.length > 0 ? "Nearby occurrences" : "Nearest occurrences";
  const subheading = document.createElement("p");
  subheading.className = "inspector-subheading";
  subheading.textContent = `${world.deposits.length.toLocaleString()} mapped systems · ${new Set(world.deposits.map((deposit) => deposit.typeId)).size} deposit types`;
  header.append(heading, subheading);
  fragment.append(header);
  for (const resource of shown) fragment.append(resourceItem(resource));
  container.replaceChildren(fragment);
}

function resourceItem({ deposit, distanceKm }: NearbyResource): HTMLElement {
  const item = document.createElement("article");
  item.className = "resource-item";
  const heading = document.createElement("h3");
  heading.textContent = deposit.name;
  const meta = document.createElement("div");
  meta.className = "resource-meta";
  meta.textContent = `${Math.round(distanceKm).toLocaleString()} km · ${Math.round(deposit.potential * 100)}% potential · ${Math.round(deposit.quality * 100)}% quality`;
  const setting = document.createElement("p");
  setting.textContent = `${deposit.setting}. Host: ${deposit.host}.`;
  const formation = document.createElement("p");
  formation.textContent = deposit.formation;
  const details = document.createElement("p");
  const age = deposit.ageMa < 0.1 ? "modern" : `${Math.round(deposit.ageMa).toLocaleString()} Ma`;
  details.textContent = `${deposit.commodities.join(", ")} · ${age} · ${Math.round(deposit.depthMeters).toLocaleString()} m depth · ${deposit.thicknessMeters.toFixed(1)} m thick`;
  item.append(heading, meta, setting, formation, details);
  return item;
}
