import { z } from "zod";

export const WORLD_VERSION = "0.3.0";
export const WORLD_FORMAT_ABI = 300;

export const RESOLUTION_PRESETS = {
  draft: { width: 256, height: 128 },
  standard: { width: 512, height: 256 },
  high: { width: 1024, height: 512 }
} as const;

export type ResolutionPreset = keyof typeof RESOLUTION_PRESETS;

export const WorldRecipeSchema = z
  .object({
    seed: z.number().int().min(0).max(0xffffffff),
    width: z.number().int().min(128).max(1024),
    height: z.number().int().min(64).max(512),
    radiusKm: z.number().finite().min(2500).max(12000),
    worldAgeGa: z.number().finite().min(0.5).max(10),
    plateCount: z.number().int().min(6).max(48),
    continentalFraction: z.number().finite().min(0.15).max(0.68),
    tectonicActivity: z.number().finite().min(0.2).max(2),
    volcanism: z.number().finite().min(0).max(2),
    seaLevelMeters: z.number().finite().min(-1500).max(1500),
    erosionCycles: z.number().int().min(2).max(48),
    erosionStrength: z.number().finite().min(0.1).max(2.5),
    sedimentTransport: z.number().finite().min(0.1).max(2.5),
    riverMobility: z.number().finite().min(0.1).max(2.5),
    axialTiltDegrees: z.number().finite().min(0).max(45),
    solarConstantWm2: z.number().finite().min(900).max(1800),
    greenhouseOffsetC: z.number().finite().min(-20).max(30),
    moistureTransport: z.number().finite().min(0.15).max(2.5),
    orographicStrength: z.number().finite().min(0.1).max(2.5),
    oceanHeatTransport: z.number().finite().min(0.1).max(2.5),
    glaciation: z.number().finite().min(0).max(2.5),
    soilMaturity: z.number().finite().min(0.1).max(2.5),
    vegetationProductivity: z.number().finite().min(0.1).max(2.5),
    mineralAbundance: z.number().finite().min(0.1).max(2.5),
    organicResourceAbundance: z.number().finite().min(0.1).max(2.5),
    spice: z.number().finite().min(0).max(1)
  })
  .superRefine((value, context) => {
    if (value.width !== value.height * 2) {
      context.addIssue({ code: z.ZodIssueCode.custom, path: ["width"], message: "World fields require a 2:1 longitude/latitude grid." });
    }
    if (value.width % 64 !== 0) {
      context.addIssue({ code: z.ZodIssueCode.custom, path: ["width"], message: "World width must align to 64-cell GPU workgroups." });
    }
  });

export type WorldRecipe = z.infer<typeof WorldRecipeSchema>;

export const DEFAULT_RECIPE: WorldRecipe = {
  seed: 731942,
  width: RESOLUTION_PRESETS.standard.width,
  height: RESOLUTION_PRESETS.standard.height,
  radiusKm: 6371,
  worldAgeGa: 4.55,
  plateCount: 16,
  continentalFraction: 0.38,
  tectonicActivity: 1,
  volcanism: 1,
  seaLevelMeters: 0,
  erosionCycles: 18,
  erosionStrength: 1,
  sedimentTransport: 1,
  riverMobility: 1,
  axialTiltDegrees: 23.44,
  solarConstantWm2: 1361,
  greenhouseOffsetC: 0,
  moistureTransport: 1,
  orographicStrength: 1,
  oceanHeatTransport: 1,
  glaciation: 1,
  soilMaturity: 1,
  vegetationProductivity: 1,
  mineralAbundance: 1,
  organicResourceAbundance: 1,
  spice: 0.35
};

export function normalizeRecipe(recipe: Partial<WorldRecipe>): WorldRecipe {
  return WorldRecipeSchema.parse({ ...DEFAULT_RECIPE, ...recipe });
}

export function recipeForPreset(recipe: WorldRecipe, preset: ResolutionPreset): WorldRecipe {
  return WorldRecipeSchema.parse({ ...recipe, ...RESOLUTION_PRESETS[preset] });
}
