# Project Bombo Schema Index

This folder documents the contracts implemented by Project Bombo `0.2.0`. The source of truth is the runtime schema in `src/schema`; these documents explain intent, ownership, algorithms, limitations, and acceptance rules without assigning work to backends that do not currently perform it.

The authoritative simulation is TypeScript running in a worker, the authoritative renderer is Canvas 2D, Rust/WASM provides a narrow deterministic grid preflight, and WebGPU provides an optional elevation diagnostic.

## Reading Order

1. [01-product-frame.md](01-product-frame.md) - product identity, audience, and scope.
2. [02-source-concepts.md](02-source-concepts.md) - research concepts and their implementation status.
3. [03-schema-driven-workflow.md](03-schema-driven-workflow.md) - how contracts drive implementation.
4. [04-domain-object-model.md](04-domain-object-model.md) - canonical runtime objects and units.
5. [05-simulation-pipeline.md](05-simulation-pipeline.md) - ordered generation and invalidation.
6. [06-region-and-planet-grid-schema.md](06-region-and-planet-grid-schema.md) - spherical graph representation.
7. [07-tectonics-schema.md](07-tectonics-schema.md) - plates, motion, contacts, and terrain influence.
8. [08-terrain-erosion-hydrology-schema.md](08-terrain-erosion-hydrology-schema.md) - elevation, erosion, and drainage.
9. [09-climate-biomes-resources-schema.md](09-climate-biomes-resources-schema.md) - climate, classification, and suitability.
10. [10-rendering-webgpu-schema.md](10-rendering-webgpu-schema.md) - Canvas rendering and optional GPU diagnostics.
11. [11-rust-wasm-modules-schema.md](11-rust-wasm-modules-schema.md) - current Rust/WASM ABI and fallback.
12. [12-typescript-app-architecture.md](12-typescript-app-architecture.md) - module boundaries and orchestration.
13. [13-ui-ux-schema.md](13-ui-ux-schema.md) - interaction and interface contracts.
14. [14-data-contracts-and-storage.md](14-data-contracts-and-storage.md) - persistence, migration, and export.
15. [15-quality-validation-and-done.md](15-quality-validation-and-done.md) - validation and release gates.
16. [16-implementation-roadmap.md](16-implementation-roadmap.md) - completed work and remaining priorities.
17. [17-recursive-document-review.md](17-recursive-document-review.md) - historical document connection audit.

## Contract Rule

A feature is accepted only when its data shape is typed, external data is validated, ownership is explicit, failure is visible, and behavior is covered at the risk-appropriate level. Proposed work must be labeled as proposed rather than described as current behavior.
