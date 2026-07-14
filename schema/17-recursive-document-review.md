# Recursive Document Review

This file records the required second pass: each schema document was reviewed for whether its concepts should be expanded or connected more strongly to other documents.

## Review Method

For each file:

1. Read the document.
2. Identify its primary concepts.
3. Check whether those concepts have upstream sources and downstream consumers.
4. Add cross-links where concepts span files.
5. Note any remaining expansion candidate that belongs in implementation, not this schema pack.

## Per-Document Review

### 00-index.md

Status: adequate as the navigation root.

Expansion added elsewhere: later documents now link back to each other, so the index is no longer the only navigation surface.

### 01-product-frame.md

Concepts: product type, audience, non-goals, causal promise.

Expansion: added schema connections to source concepts, pipeline, domain model, UI, and done criteria. The product promise now has explicit implementation homes.

### 02-source-concepts.md

Concepts: Bombo research spine, World-Synth pattern, region model, tectonic approximation, WebGPU, Rust/WASM, open questions.

Expansion: added links from each research concept to the schema file that operationalizes it.

### 03-schema-driven-workflow.md

Concepts: schemas as source of truth, development loop, change control.

Expansion: added a recursive review rule so future schema edits must be checked against upstream and downstream documents.

### 04-domain-object-model.md

Concepts: canonical objects and anti-ad-hoc data rule.

Expansion: added connections to the detailed region, tectonics, terrain, climate, storage, and architecture schemas.

### 05-simulation-pipeline.md

Concepts: stage contract, ordered generation, invalidation.

Expansion: added links from stage ranges to the detailed schemas and runtime architecture.

### 06-region-and-planet-grid-schema.md

Concepts: spherical graph, region attributes, spatial index, GPU layout.

Expansion: added links to domain model, pipeline stage, tectonics, downstream layers, WebGPU, and Rust/WASM ownership.

### 07-tectonics-schema.md

Concepts: plates, plate growth, boundaries, geology honesty.

Expansion: added links to region ownership, terrain/climate outputs, rendering overlays, Rust/WASM graph work, and validation gates.

### 08-terrain-erosion-hydrology-schema.md

Concepts: terrain fields, water fields, erosion passes, race-condition constraints.

Expansion: added links to upstream tectonics and regions, downstream climate/biomes, WebGPU rules, Rust/WASM hydrology, and validation gates.

### 09-climate-biomes-resources-schema.md

Concepts: climate, biomes, resource potential, suitability scoring.

Expansion: added links to terrain/hydrology, tectonics, region outputs, overlays, UI, and Rust/WASM classification.

### 10-rendering-webgpu-schema.md

Concepts: GPU buffers, render modes, inspectability, fallback.

Expansion: added links to domain fields, region buffers, simulation overlays, TypeScript device setup, UI legends, and validation.

### 11-rust-wasm-modules-schema.md

Concepts: CPU-heavy deterministic modules, ABI rules, error taxonomy.

Expansion: added module-to-schema mapping for grid, tectonics, hydrology, classification, TypeScript wrappers, and storage.

### 12-typescript-app-architecture.md

Concepts: app shell, state, commands, schema mirrors.

Expansion: added links from source folders to product, pipeline, schema, GPU, WASM, UI, and validation documents.

### 13-ui-ux-schema.md

Concepts: causality-first UI, screens, controls, visual rules.

Expansion: added connections to product promise, domain objects, pipeline, inspectors, rendering overlays, storage flows, and UI done states.

### 14-data-contracts-and-storage.md

Concepts: manifests, layer references, versioning, exports, provenance.

Expansion: added links to domain objects, pipeline stage statuses, Rust/WASM serialization, TypeScript storage modules, UI flows, and round-trip validation.

### 15-quality-validation-and-done.md

Concepts: done criteria, schema done, feature done, simulation done, UI done, terrain realism research gate.

Expansion: added recursive review done criteria.

### 16-implementation-roadmap.md

Concepts: phased development sequence.

Expansion: added recursive review as part of Phase 0 schema lock.

## Remaining Expansion Candidates

These are intentionally not expanded into full implementation specs yet:

- Exact TypeScript type definitions.
- Exact WGSL struct layouts.
- Exact Rust crate APIs.
- Exact UI component library choice.
- Exact terrain realism metrics.

They should be created during Phase 0 and Phase 1 using the workflow in [03-schema-driven-workflow.md](03-schema-driven-workflow.md).

## Review Conclusion

The first draft was conceptually coherent but under-linked. The second pass made the schema recursive: product goals point to implementation schemas, implementation schemas point to runtime ownership, and validation points back to the cross-document audit.
