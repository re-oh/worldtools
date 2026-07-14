# Implementation Status And Roadmap

## Completed In `0.2.0`

- Runtime-validated domain, parameter, pipeline, storage, and GPU layouts.
- Deterministic equal-area regional graph with wrapped adjacency.
- Connected plates, explicit Euler motion, boundary kinematics, and multi-ring terrain influence.
- Terrain decomposition, thermal erosion, priority-flood drainage, incision, deposition, and basins.
- Coast-distance climate, orographic rainfall, biomes, resource signals, and use-specific suitability.
- Worker generation with progress, cancellation, stale-stage invalidation, and explicit on-demand stages.
- Interactive semantic map, thirteen overlays, cartographic features, selection, comparison, and inspectors.
- Local library, validated archive import/export, checksum verification, CSV, manifest, and PNG export.
- Narrow Rust/WASM ABI with fallback and optional WebGPU full-world diagnostic.
- Domain-focused test suite, Rust unit tests, modular UI/render/state/simulation code, and responsive styles.

## Next Engineering Priorities

1. Add automated viewport screenshots and interaction regression once a supported browser harness is available.
2. Measure generation time and memory at draft, standard, and detailed resolutions; establish budgets before optimization.
3. Add IndexedDB repository tests against a browser implementation, including invalid-record recovery behavior.
4. Add archive migrations keyed by explicit source schema instead of regeneration-only handling.
5. Profile the worker and move only proven graph bottlenecks behind typed-array WASM interfaces.

## Next Model Priorities

1. Replace the rectangular spherical graph with a lower-distortion mesh while preserving stable lookup and migration.
2. Represent endorheic lakes and spill elevations explicitly.
3. Add conservative water/sediment transport and stream hierarchy.
4. Advect moisture over multiple cells and model seasonal temperature/rainfall ranges.
5. Define external terrain metrics for slopes, drainage density, basin geometry, and mountain continuity.

Future work must preserve deterministic replay, explicit units, inspectable provenance, and truthful backend reporting.
