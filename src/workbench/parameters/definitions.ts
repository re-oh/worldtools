import type { WorldRecipe } from "../../world/recipe";
import type { WorldStageId } from "../../world/model";

export type ParameterGroup = "World" | "Tectonics" | "Surface" | "Climate" | "Ecology" | "Resources";
export type EditableRecipeKey = Exclude<keyof WorldRecipe, "width" | "height">;

export interface ParameterDefinition {
  key: EditableRecipeKey;
  label: string;
  group: ParameterGroup;
  minimum: number;
  maximum: number;
  step: number;
  invalidates: WorldStageId;
  format?: (value: number) => string;
  control?: "slider" | "number";
}

const percent = (value: number) => `${Math.round(value * 100)}%`;
const fixed = (digits: number) => (value: number) => value.toFixed(digits);

export const PARAMETER_DEFINITIONS: readonly ParameterDefinition[] = [
  { key: "seed", label: "Seed", group: "World", minimum: 0, maximum: 0xffffffff, step: 1, invalidates: "geology", control: "number" },
  { key: "radiusKm", label: "Planet radius", group: "World", minimum: 2500, maximum: 12000, step: 10, invalidates: "geology", format: (value) => `${Math.round(value)} km` },
  { key: "worldAgeGa", label: "World age", group: "World", minimum: 0.5, maximum: 10, step: 0.05, invalidates: "geology", format: (value) => `${value.toFixed(2)} Ga` },
  { key: "plateCount", label: "Plate count", group: "World", minimum: 6, maximum: 48, step: 1, invalidates: "geology" },
  { key: "continentalFraction", label: "Continental crust", group: "World", minimum: 0.15, maximum: 0.68, step: 0.01, invalidates: "geology", format: percent },
  { key: "seaLevelMeters", label: "Sea level offset", group: "World", minimum: -1500, maximum: 1500, step: 25, invalidates: "terrain", format: (value) => `${Math.round(value)} m` },
  { key: "tectonicActivity", label: "Tectonic activity", group: "Tectonics", minimum: 0.2, maximum: 2, step: 0.05, invalidates: "geology", format: fixed(2) },
  { key: "volcanism", label: "Mantle volcanism", group: "Tectonics", minimum: 0, maximum: 2, step: 0.05, invalidates: "tectonics", format: fixed(2) },
  { key: "spice", label: "Geological variation", group: "Tectonics", minimum: 0, maximum: 1, step: 0.01, invalidates: "tectonics", format: percent },
  { key: "erosionCycles", label: "Landscape epochs", group: "Surface", minimum: 2, maximum: 48, step: 1, invalidates: "hydrology" },
  { key: "erosionStrength", label: "Erosion intensity", group: "Surface", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "cryosphere", format: fixed(2) },
  { key: "sedimentTransport", label: "Sediment transport", group: "Surface", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "hydrology", format: fixed(2) },
  { key: "riverMobility", label: "River mobility", group: "Surface", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "hydrology", format: fixed(2) },
  { key: "soilMaturity", label: "Soil maturity", group: "Surface", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "soils", format: fixed(2) },
  { key: "axialTiltDegrees", label: "Axial tilt", group: "Climate", minimum: 0, maximum: 45, step: 0.25, invalidates: "ocean", format: (value) => `${value.toFixed(2)}°` },
  { key: "solarConstantWm2", label: "Stellar flux", group: "Climate", minimum: 900, maximum: 1800, step: 5, invalidates: "ocean", format: (value) => `${Math.round(value)} W/m²` },
  { key: "greenhouseOffsetC", label: "Greenhouse offset", group: "Climate", minimum: -20, maximum: 30, step: 0.5, invalidates: "climate", format: (value) => `${value.toFixed(1)} °C` },
  { key: "moistureTransport", label: "Atmospheric moisture", group: "Climate", minimum: 0.15, maximum: 2.5, step: 0.05, invalidates: "climate", format: fixed(2) },
  { key: "orographicStrength", label: "Orographic forcing", group: "Climate", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "climate", format: fixed(2) },
  { key: "oceanHeatTransport", label: "Ocean heat transport", group: "Climate", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "ocean", format: fixed(2) },
  { key: "glaciation", label: "Glacial forcing", group: "Climate", minimum: 0, maximum: 2.5, step: 0.05, invalidates: "cryosphere", format: fixed(2) },
  { key: "vegetationProductivity", label: "Plant productivity", group: "Ecology", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "ecology", format: fixed(2) },
  { key: "mineralAbundance", label: "Mineral occurrence", group: "Resources", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "resources", format: fixed(2) },
  { key: "organicResourceAbundance", label: "Organic occurrence", group: "Resources", minimum: 0.1, maximum: 2.5, step: 0.05, invalidates: "resources", format: fixed(2) }
] as const;

export const PARAMETER_DEFINITIONS_BY_KEY = Object.fromEntries(
  PARAMETER_DEFINITIONS.map((definition) => [definition.key, definition])
) as Record<EditableRecipeKey, ParameterDefinition>;

export const PARAMETER_GROUPS: readonly ParameterGroup[] = ["World", "Tectonics", "Surface", "Climate", "Ecology", "Resources"];
