# TypeScript Application Architecture

Project Bombo uses explicit domain modules and a thin coordination layer. Dependencies point from app/UI adapters toward schema, simulation, state, rendering, and validation contracts; simulation code does not depend on the DOM.

## Module Boundaries

```text
src/app/
  AppShell.ts                 world/session orchestration
  AppView.ts                  stable DOM layout updates
  AppEventController.ts       delegated DOM event decoding
  MapInteractionController.ts pointer, wheel, keyboard, resize
  layout.ts, types.ts         data-driven shell descriptors and view models
src/simulation/
  grid.ts, tectonics.ts, terrain.ts, climate.ts, classification.ts
  pipelineRunner.ts, layers.ts, math.ts, invalidation.ts
  simulationClient.ts, simulation.worker.ts, workerProtocol.ts
src/gpu/
  renderer.ts, mapView.ts, rasterizer.ts, cartography.ts
  device.ts, passes/elevationCompute.ts
src/state/
  parameterDraft.ts, worldRepository.ts, archiveCodec.ts, csvExport.ts
src/schema/                  runtime-validated contracts
src/ui/                      controls, inspectors, overlays, panels
src/validation/              invariants, metrics, fixtures
```

Styles follow the same ownership split: base tokens, shell, map, panels, pipeline, and responsive behavior are separate imports.

## State Rules

- A `World` is immutable at module boundaries; stages return copied/derived arrays.
- `ParameterDraft` separates applied parameters from edits and derives stale stages.
- Only `AppShell` installs a generated/loaded world and coordinates selection.
- `AppView` renders typed view models and does not run domain logic.
- DOM attribute parsing lives in `AppEventController`; map gestures live in `MapInteractionController`.
- Long generation runs in one disposable worker and resolves only complete worlds.
- Repository and codec functions validate external/persisted objects before returning them.

## Extension Rules

A new simulation stage belongs in the stage registry, typed pipeline, invalidation map, layer catalog, validation, UI provenance, and tests. A new overlay is one catalog entry plus any specialized cartography, not a conditional spread across the app. A new export belongs in a codec/module and is only orchestrated by the shell.

Abstractions should follow stable responsibilities. File splitting solely to reduce line counts is not a goal; separating algorithms, I/O, interaction decoding, and presentation is.
