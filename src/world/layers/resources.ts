import type { FieldId } from "../model";
import { PALETTES as P } from "./palettes";
import { defineLayer } from "./types";

const labels: Readonly<Record<FieldId, string>> = {
  plateId: "", continentalness: "", crustAge: "", crustThickness: "", convergence: "", divergence: "", shear: "", uplift: "", volcanism: "", sedimentaryBasin: "", carbonatePlatform: "", lithology: "", elevation: "", waterDepth: "", lakeDepth: "", slope: "", erosion: "", sediment: "", oceanCurrent: "", upwelling: "", seaSurfaceTemperature: "", flow: "", runoff: "", floodplain: "", channelMobility: "", delta: "", ice: "", glacialErosion: "", glacialDeposit: "", temperature: "", temperatureSeasonality: "", precipitation: "", aridity: "", windSpeed: "", storminess: "", freezeThaw: "", weathering: "", regolithDepth: "", regolithMoisture: "", clayFraction: "", organicCarbon: "", soilPH: "", soilType: "", mineralFertility: "", texture: "", vegetationCover: "", forestCover: "", grassCover: "", wetlandCover: "", biome: "",
  bandedIron: "Banded iron systems", bauxite: "Bauxite systems", copperSulfide: "Copper sulfide systems", magmaticSulfide: "Magmatic sulfides", carbonatite: "Carbonatite complexes", orogenicGold: "Orogenic gold systems", copper: "Copper", gold: "Gold", iron: "Iron", rareEarth: "Rare earths", nickel: "Nickel", placer: "Placer concentration", mineralPotential: "Metallic mineral systems", clayDeposit: "Clay deposits", peat: "Peat", coal: "Coal", hydrocarbon: "Oil and gas", evaporite: "Evaporites and salt", nitrate: "Nitrate caliche", gemstone: "Gemstones", resourcePotential: "All natural resources"
};

const mineralIds = ["bandedIron", "bauxite", "copperSulfide", "magmaticSulfide", "carbonatite", "orogenicGold", "copper", "gold", "iron", "rareEarth", "nickel", "placer", "mineralPotential"] as const;
const otherIds = ["clayDeposit", "peat", "coal", "hydrocarbon", "evaporite", "nitrate", "gemstone", "resourcePotential"] as const;

export const RESOURCE_LAYERS = [
  ...mineralIds.map((id) => defineLayer(id, labels[id], "Resources", "resources", "potential", "log", P.mineral, { fixedRange: [0, 1] })),
  ...otherIds.map((id) => defineLayer(id, labels[id], "Resources", "resources", "potential", "log", id === "peat" || id === "coal" || id === "hydrocarbon" ? P.organic : P.earth, { fixedRange: [0, 1] }))
] as const;
