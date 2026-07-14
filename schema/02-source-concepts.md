# Source Concepts From Bombo

This file records the research concepts that shape the app schema.

## Scientific-Fantasy World Simulator

The Bombo corpus describes a genre where fantasy world generation is grounded in scientific models. Required simulation domains:

- Plate tectonics.
- Terrain and orogeny.
- Surface water movement.
- Hydraulic and thermal erosion.
- Climate and rainfall.
- Biome classification.
- Glaciers and fjords.
- Magma cooling and rock classification.
- Resource and land suitability.

## World-Synth Pattern

World-Synth contributes the core performance pattern:

- Offline phase: precompute region-scale features.
- Online phase: query compact region data while streaming terrain chunks.
- Planet grid: approximately tens of thousands of regions, each with neighbors, center coordinates, plate ownership, and crust type.
- Plate growth: flood-fill or priority-queue growth driven by cost functions.

## Region Concept

A region is the atomic simulation cell. It must:

- Know its neighbors.
- Be queryable by latitude/longitude or 3D vector.
- Store plate ownership.
- Store crust type.
- Link to generated terrain, climate, biome, resource, and suitability outputs.

## Tectonic Terrain Simulation

Bombo separates real geology from graphics-engineering approximation. The app must preserve that honesty:

- Use plate tectonics as a rule system for plausible terrain.
- Avoid claiming full geophysical fidelity.
- Name project-specific approximations separately from geological terms.
- Map terms like "fault point" to more precise internal terms such as boundary segment, perturbation point, or displacement control.

## WebGPU Compute

The research suggests WebGPU for profiled data-parallel work, but the current simulation remains CPU-based. WebGPU `0.2.0` runs one explicit 64-thread elevation diagnostic. Erosion, water flow, and terrain rendering must not be described as GPU-backed unless future implementations and provenance actually make them so.

## Rust/WASM

Rust/WASM is appropriate for measured CPU-heavy graph work with a stable flat ABI. The current module owns deterministic seed expansion and grid preflight only. Plate growth, priority queues, spatial traversal, classification, storage, and export remain TypeScript responsibilities; they are migration candidates rather than implemented Rust modules.

## Open Questions To Encode

The schema must leave room for research questions:

- How to measure generated terrain realism objectively.
- How to compare hidden or external algorithms such as Gleba's plate model.
- How to distinguish geologic terminology from internal model terminology.
- How to mark provisional science claims versus validated claims.

## Schema Connections

- World-Synth's offline/online idea becomes the stage architecture in [05-simulation-pipeline.md](05-simulation-pipeline.md).
- Region requirements become concrete fields and invariants in [06-region-and-planet-grid-schema.md](06-region-and-planet-grid-schema.md).
- Tectonic approximation rules become the geology honesty rules in [07-tectonics-schema.md](07-tectonics-schema.md).
- WebGPU concepts become buffer and pass rules in [10-rendering-webgpu-schema.md](10-rendering-webgpu-schema.md).
- Rust/WASM concepts become module boundaries in [11-rust-wasm-modules-schema.md](11-rust-wasm-modules-schema.md).
- The unresolved realism and terminology questions become validation gates in [15-quality-validation-and-done.md](15-quality-validation-and-done.md).

## Finish Definition

This file is done when every major Bombo concept has a matching schema home in later files.
