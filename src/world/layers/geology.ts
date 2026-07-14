import { PALETTES as P } from "./palettes";
import { defineLayer } from "./types";

export const GEOLOGY_LAYERS = [
  defineLayer("plateId", "Tectonic plates", "Geology", "geology", "id", "categorical", P.terrain),
  defineLayer("continentalness", "Continental affinity", "Geology", "geology", "index", "diverging", P.stress, { fixedRange: [-1, 1] }),
  defineLayer("crustAge", "Crust age", "Geology", "tectonics", "Ma", "continuous", P.earth),
  defineLayer("crustThickness", "Crust thickness", "Geology", "geology", "km", "continuous", P.earth, { fixedRange: [5, 65] }),
  defineLayer("convergence", "Convergence", "Geology", "tectonics", "index", "continuous", P.heat, { fixedRange: [0, 1] }),
  defineLayer("divergence", "Divergence", "Geology", "tectonics", "index", "continuous", P.water, { fixedRange: [0, 1] }),
  defineLayer("shear", "Transform shear", "Geology", "tectonics", "index", "continuous", P.stress, { fixedRange: [0, 1] }),
  defineLayer("uplift", "Orogenic uplift", "Geology", "tectonics", "index", "continuous", P.heat, { fixedRange: [0, 1] }),
  defineLayer("volcanism", "Magmatic activity", "Geology", "tectonics", "index", "continuous", P.heat, { fixedRange: [0, 1] }),
  defineLayer("sedimentaryBasin", "Sedimentary basins", "Geology", "tectonics", "index", "continuous", P.earth, { fixedRange: [0, 1] }),
  defineLayer("carbonatePlatform", "Carbonate platforms", "Geology", "tectonics", "index", "continuous", P.earth, { fixedRange: [0, 1] }),
  defineLayer("lithology", "Lithology", "Geology", "tectonics", "class", "categorical", P.earth, {
    categories: { 0: "Oceanic basalt", 1: "Mafic volcanic", 2: "Felsic intrusive", 3: "Metamorphic belt", 4: "Sedimentary basin", 5: "Ultramafic / ophiolite" }
  })
] as const;
