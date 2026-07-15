# Study Plan

This plan turns the repository into a sequence of questions. It is more effective than reading every file top to bottom.

## 30 minutes: product and boundaries

Read:

1. [Getting Started](getting-started.md)
2. [Architecture](architecture.md), through the ownership section
3. [`Cargo.toml`](../../Cargo.toml)
4. [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs)

Be able to answer:

- What natural-world problem does the program solve?
- Why are world, simulation, render, UI, and application separate crates?
- Which crate is allowed to know about both UI and rendering?
- What is deterministic input and what is derived output?

## 60 minutes: one generated world

Read:

1. [Simulation](simulation.md)
2. [`settings.rs`](../../crates/worldtools_simulation/src/settings.rs)
3. [`stages/mod.rs`](../../crates/worldtools_simulation/src/stages/mod.rs)
4. [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs)
5. [Data Layers](data-layers.md)

Write down the stage graph without looking. Then identify one concrete coupling for every arrow. Example: topography changes orographic precipitation; precipitation changes discharge; discharge changes erosion and sediment.

## 60 minutes: one rendered frame

Read:

1. [Rendering and UI](rendering-ui.md)
2. [`view.rs`](../../crates/worldtools_render/src/view.rs)
3. [`projection.rs`](../../crates/worldtools_render/src/projection.rs)
4. [`streaming.rs`](../../crates/worldtools_render/src/streaming.rs)
5. [`tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs)
6. [`tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs)
7. The final `fragment` function in [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl), then its helpers

Be able to draw:

```text
MapView -> TilePlan -> async request -> MapTileData -> Image -> Material -> fragment color
```

Explain why a visible placement has both a canonical ID and `unwrapped_x`.

## 45 minutes: user intent and application integration

Read:

1. [`model/tools.rs`](../../crates/worldtools_ui/src/model/tools.rs)
2. [`model/presentation.rs`](../../crates/worldtools_ui/src/model/presentation.rs)
3. [`shell/explorer.rs`](../../crates/worldtools_ui/src/shell/explorer.rs)
4. [`shell/inspector.rs`](../../crates/worldtools_ui/src/shell/inspector.rs)
5. [`apps/worldtools/src/viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs)
6. [`apps/worldtools/src/generation.rs`](../../apps/worldtools/src/generation.rs)

Be able to answer:

- Which UI changes are presentation-only?
- Which action replaces the immutable snapshot?
- Why are UI and simulation layer enums translated explicitly?
- How is stale asynchronous work rejected?

## 45 minutes: evidence-driven debugging

Read [Debugging](debugging.md), then inspect:

1. [`xtask/src/cli.rs`](../../xtask/src/cli.rs)
2. [`xtask/src/reproduce.rs`](../../xtask/src/reproduce.rs)
3. [`apps/worldtools/src/diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs)
4. [`apps/worldtools/src/debug_tools`](../../apps/worldtools/src/debug_tools)
5. [`crates/worldtools_analysis/src`](../../crates/worldtools_analysis/src)

Choose one hypothetical defect and write:

- deterministic reproduction
- three or fewer hypotheses
- cheapest falsifying experiment
- evidence to retain
- focused verification after the fix

## Half-day source exercises

These exercises require no code changes.

### Exercise A: predict a layer page

Pick hydrology. From [`layers.rs`](../../crates/worldtools_simulation/src/layers.rs), write the four GPU channels and their units/normalization. Then find each source value in the typed snapshot.

### Exercise B: explain a layer switch

Starting at an explorer click, list every type and function involved until the new GPU page is visible. Include the temporary physical fallback behavior.

### Exercise C: prove determinism

Find the tests that verify identical generation and sampling for the same seed. Explain which kinds of nondeterminism those tests do and do not catch.

### Exercise D: classify a seam

Use the analysis, tile-data, and renderer-debug modules to distinguish:

- cube-face seam
- map-page data seam
- parent/child LOD seam
- shader footprint seam
- stale fallback mismatch

### Exercise E: trace one deposit

Pick banded iron formation, bauxite, porphyry copper, petroleum, salt, or nitrate. Start in the resource stage, list the geological/environmental conditions, follow the result into snapshot sampling, probe text, GPU channels, palette, and legend.

## Confidence checklist

You are ready to make substantial changes when you can answer all of these without searching:

- Where is authoritative generated state stored?
- What is the exact simulation stage order?
- Which fields are categorical and must never be interpolated?
- How does the flat map wrap without duplicating generated page data?
- How are exact, fallback, stale, and missing pages distinguished?
- Which visual controls modify uniforms only?
- Where do logs, snapshots, and captured cases go?
- Which tests protect seams, determinism, and layer contracts?

