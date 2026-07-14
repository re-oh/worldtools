# Simulation Pipeline

`src/simulation/pipelineRunner.ts` is the orchestration boundary. Normalized parameters enter once, ten deterministic stages execute in order, and a schema-valid `World` leaves once. Generation normally runs in a dedicated worker and reports stage-level progress to the UI.

## Ordered Stages

| ID | Output | Current backend |
| --- | --- | --- |
| 01 Seed and planet | validated parameters, seed hash, planet constants | Mixed WASM/fallback + CPU |
| 02 Region grid | equal-area wrapped region graph | Mixed WASM/fallback preflight + CPU construction |
| 03 Plate seeding | separated origins and crust biases | CPU |
| 04 Plate growth | connected plate ownership and crust type | CPU |
| 05 Plate motion | Euler poles and angular velocities | CPU |
| 06 Boundary classification | contacts, convergence, shear, hazard potentials | CPU |
| 07 Base terrain | base, tectonic, surface, and initial water elevation | CPU |
| 08 Erosion and hydrology | erosion, deposition, downstream graph, basins, discharge | CPU |
| 09 Climate | coast distance, temperature, rain, aridity, refined runoff | CPU |
| 10 Land systems | biomes, resources, explained suitability | CPU |
| 11 Render products | cached overlay raster and optional GPU diagnostic | On demand, Canvas CPU + optional WebGPU |
| 12 Export and replay | archive, manifest, CSV, PNG, checksum | On demand, CPU |

Stages 11 and 12 are recorded as `skipped` during generation because their products are requested later. They are not reported as completed simulation work.

## Failure And Progress

The runner emits `running`, then `fresh` or `failed`, for each executable stage. It records measured durations but does not use them for deterministic identity. The worker can be terminated; the active stage becomes `cancelled`, and partial worlds are never installed as current state.

## Invalidation

| Parameter | First invalidated stage |
| --- | --- |
| `seed` | 01 |
| `latBands`, `lonBands` | 02 |
| `plateCount` | 03 |
| `seaLevelMeters` | 07 |
| `erosionIterations` | 08 |
| `useCaseId` | 10 |

Draft parameters mark that stage and every downstream stage stale. A new world is generated only on explicit apply/generate.

## Acceptance

Generation succeeds only if the final world passes Zod parsing. Runtime validations cover topology, plates, fields, drainage, classification, and provenance; archive replay adds checksum verification.
