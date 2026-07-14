# Project Bombo

Project Bombo is a local-first planetary systems workbench. It generates a deterministic spherical region graph, derives tectonics, terrain, drainage, climate, biomes, resources, and use-specific suitability, then exposes every layer through an interactive technical map.

The current release is `0.2.0`. It is a plausibility model for exploration and testing, not a geodynamic solver, weather forecast, or land-use authority.

## What Works

- Deterministic generation from seed and normalized parameters.
- Equal-area latitude sampling with a wrapped eight-neighbor spherical graph.
- Connected priority-frontier plate growth and explicit Euler-pole motion.
- Boundary convergence, divergence, shear, uplift, trench, volcanic, and earthquake signals.
- Seam-safe spherical terrain noise, thermal erosion, depression filling, river incision, deposition, and drainage basins.
- Coast-distance climate, circulation bands, windward/leeward precipitation, biome classification, resource signals, and six suitability use cases.
- Thirteen inspectable map overlays with units, source-stage provenance, legends, coastlines, rivers, and tectonic contacts.
- Region, plate, comparison, pipeline, validation, runtime, and storage views.
- Cancellable worker generation, IndexedDB saves, checksummed archive import/export, manifest export, CSV export, and PNG snapshots.
- Optional Rust/WASM topology preflight and optional WebGPU elevation diagnostics with explicit fallbacks.

## Architecture

```text
src/
  app/          application coordination, view composition, DOM and map input controllers
  schema/       Zod-backed domain, parameter, pipeline, storage, and GPU contracts
  simulation/   grid, tectonics, terrain, climate, classification, layers, worker runtime
  gpu/          Canvas map renderer, cartography, view math, raster cache, WebGPU diagnostic
  state/        parameter drafts, archives, CSV, IndexedDB repository
  ui/           reusable controls, inspectors, overlays, and panels
  validation/   invariants, metrics, and fixtures
  styles/       focused layout, map, panel, pipeline, and responsive stylesheets
crates/
  bombo_core/   small deterministic Rust ABI compiled to WebAssembly
tests/          behavior tests grouped by domain contract
schema/         source-derived design and implementation documentation
```

`AppShell` coordinates these modules. It does not own simulation algorithms, persistence codecs, map interaction math, DOM event decoding, or presentation markup.

## Runtime Model

The ten executable simulation stages run on a dedicated web worker. Stages 11 and 12 are on-demand render/export products and are reported as skipped during generation rather than falsely marked as compute work.

The authoritative simulation backend is currently TypeScript/CPU. Rust/WASM supplies deterministic seed and grid contract functions when the compiled module loads; a behaviorally compatible TypeScript fallback keeps the app functional when it does not. Canvas 2D is the authoritative renderer. WebGPU runs a full-world elevation normalization and finite-value diagnostic when available, but it is not required to view or generate a world.

## Commands

Prerequisites: Node.js 20+, npm, Rust, and the `wasm32-unknown-unknown` target for production builds.

```powershell
npm install
npm run dev
npm test
npm run typecheck
cargo test --manifest-path crates/bombo_core/Cargo.toml
npm run build
```

`npm run build` compiles the Rust crate, copies the WASM artifact to `public/wasm`, type-checks TypeScript, and builds the Vite application.

## Persistence

Worlds are validated with the runtime schema before storage. Native archives include a manifest, schema/app versions, parameters, provenance, the complete world, and a deterministic checksum. Current-schema checksum mismatches are rejected. Older supported shapes are normalized and regenerated when their schema identity differs.

## Known Limits

- The grid cells have equal nominal area, but the rectangular graph has polar distortion and no cross-pole adjacency.
- Plate history is synthesized from present-day procedural state; it is not time-integrated mantle dynamics.
- Erosion is a stable regional approximation without explicit transient water or sediment transport fields.
- Climate uses zonal circulation, one-cell orographic sampling, and graph coast distance rather than a fluid atmosphere.
- Resources and suitability are comparative heuristic scores.
- The WASM and WebGPU modules are deliberately narrow; the UI reports their actual role and fallback state.

See [MATH_NOTES.md](MATH_NOTES.md) for the implemented model and [schema/00-index.md](schema/00-index.md) for the full contract set.
