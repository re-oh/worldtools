# Code Reading Tours

These tours are designed for reading on a train: open the linked files in order and answer the questions in each step. Do not try to understand the entire workspace from directory order.

## Tour 1: boot the native application

1. Open [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs).
   - Which plugins are unconditional?
   - Which live-debug capability is feature or environment gated?
   - Where is process-level diagnostics installed?
2. Open [`apps/worldtools/src/diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs).
   - Where do rolling logs go?
   - How are tracing events copied into the in-app event log?
3. Open [`crates/worldtools_render/src/surface.rs`](../../crates/worldtools_render/src/surface.rs).
   - Which renderer resources are initialized?
   - Which subplugins own streaming and material surfaces?
4. Open [`crates/worldtools_ui/src/lib.rs`](../../crates/worldtools_ui/src/lib.rs).
   - Notice that it exposes communication resources and messages rather than importing renderer internals.
5. Open [`crates/worldtools_ui/src/shell/mod.rs`](../../crates/worldtools_ui/src/shell/mod.rs).
   - Follow the order in which panels reserve screen space.

After this tour, you should be able to explain why the application crate exists even though renderer and UI are separate crates.

## Tour 2: follow one seed into a world

1. Open [`apps/worldtools/src/generation.rs`](../../apps/worldtools/src/generation.rs).
   - Find the regeneration message reader.
   - Identify how work is moved off the main thread and how completion is accepted.
2. Open [`crates/worldtools_simulation/src/settings.rs`](../../crates/worldtools_simulation/src/settings.rs).
   - Separate explicit world inputs from derived state.
3. Open [`crates/worldtools_simulation/src/random.rs`](../../crates/worldtools_simulation/src/random.rs) and [`crates/worldtools_world/src/seed.rs`](../../crates/worldtools_world/src/seed.rs).
   - Look for domain separation: unrelated systems should not consume one shared random stream in sequence.
4. Open [`crates/worldtools_simulation/src/stages/mod.rs`](../../crates/worldtools_simulation/src/stages/mod.rs).
   - Write down the stage order.
   - For each stage, record which earlier outputs it reads.
5. Open [`crates/worldtools_simulation/src/snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs).
   - Find generation, typed sampling, direct layer channels, and fine elevation sampling.
6. Open [`crates/worldtools_simulation/src/layers.rs`](../../crates/worldtools_simulation/src/layers.rs).
   - Map each four-channel GPU page to its typed domain data.

Checkpoint: explain why changing stage order is a model change, not a refactor.

## Tour 3: follow one visible map tile

1. Open [`crates/worldtools_render/src/view.rs`](../../crates/worldtools_render/src/view.rs).
   - Observe that horizontal position is not clamped to one world copy.
2. Open [`crates/worldtools_render/src/projection.rs`](../../crates/worldtools_render/src/projection.rs).
   - Follow viewport span into a desired LOD.
   - Follow unwrapped placement into a canonical tile ID.
3. Open [`crates/worldtools_render/src/streaming.rs`](../../crates/worldtools_render/src/streaming.rs).
   - Find request priority, in-flight work, exact pages, and ancestor fallback.
   - Note the revision and layer checks before accepting an asynchronous result.
4. Open [`crates/worldtools_render/src/tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs).
   - Follow a tile sample into `WorldSnapshot`.
   - Find the halo around the visible 256-cell page.
5. Open [`crates/worldtools_render/src/tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs).
   - Follow CPU page data into GPU images and a `TerrainTileMaterial`.
6. Open [`crates/worldtools_render/src/worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl).
   - Read the fragment function last, after its sampling, palette, lighting, and layer-view helpers.

Checkpoint: explain the difference between a desired tile, a source tile, and an unwrapped placement.

## Tour 4: follow a layer selection

1. [`crates/worldtools_ui/src/shell/explorer.rs`](../../crates/worldtools_ui/src/shell/explorer.rs): the click updates semantic editor state.
2. [`crates/worldtools_ui/src/model/editor.rs`](../../crates/worldtools_ui/src/model/editor.rs): `active_layer` is owned by the UI contract.
3. [`apps/worldtools/src/viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs): the app maps semantic layer and visual profile into renderer settings.
4. [`apps/worldtools/src/layers.rs`](../../apps/worldtools/src/layers.rs): UI and simulation layer enums are translated explicitly.
5. [`crates/worldtools_render/src/streaming.rs`](../../crates/worldtools_render/src/streaming.rs): a layer change invalidates page residency while retaining safe in-flight rules.
6. [`crates/worldtools_render/src/display.rs`](../../crates/worldtools_render/src/display.rs): visual settings are sanitized and packed for the shader.
7. [`crates/worldtools_render/src/worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl): mode selects the final presentation function.

Checkpoint: explain why the UI crate does not depend on the renderer crate.

## Tour 5: inspect a geographic point

1. [`apps/worldtools/src/interaction.rs`](../../apps/worldtools/src/interaction.rs): viewport input becomes geographic intent.
2. [`apps/worldtools/src/interaction/probe.rs`](../../apps/worldtools/src/interaction/probe.rs): the snapshot is sampled.
3. [`apps/worldtools/src/interaction/probe/format.rs`](../../apps/worldtools/src/interaction/probe/format.rs): typed values become ordered human-readable readings.
4. [`crates/worldtools_ui/src/model/probe.rs`](../../crates/worldtools_ui/src/model/probe.rs): the UI-facing selection is stored without importing simulation types.
5. [`crates/worldtools_ui/src/shell/inspector.rs`](../../crates/worldtools_ui/src/shell/inspector.rs): the reading is displayed.

Checkpoint: find where longitude wrapping occurs and where display formatting occurs. They should not be the same function.

## Tour 6: diagnose a visual seam

1. Reproduce with one seed and camera position.
2. Enable tile borders and LOD/residency tint in the diagnostics window.
3. Read [`crates/worldtools_render/src/debug.rs`](../../crates/worldtools_render/src/debug.rs).
4. Check page adjacency with [`crates/worldtools_render/src/tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs).
5. Check parent-child continuity with [`crates/worldtools_analysis/src/lod.rs`](../../crates/worldtools_analysis/src/lod.rs).
6. Compare shader footprint with `MAP_TILE_APRON` in [`projection.rs`](../../crates/worldtools_render/src/projection.rs).
7. Only then inspect interpolation and relief taps in [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl).

The key question is whether the discontinuity exists in CPU data, appears only across different source LODs, or is introduced by GPU sampling.

## Tour 7: add or change a data layer

Read these contracts before writing code:

1. Domain type and generation stage in [`crates/worldtools_simulation/src/stages`](../../crates/worldtools_simulation/src/stages)
2. Typed snapshot sampling in [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs)
3. Four-channel page contract in [`layers.rs`](../../crates/worldtools_simulation/src/layers.rs)
4. UI layer identity in [`tools.rs`](../../crates/worldtools_ui/src/model/tools.rs)
5. Explicit enum translation in [`apps/worldtools/src/layers.rs`](../../apps/worldtools/src/layers.rs)
6. Probe formatting in [`apps/worldtools/src/interaction/probe/format.rs`](../../apps/worldtools/src/interaction/probe/format.rs)
7. Shader presentation and legend
8. Determinism, variation, and finite-value tests

Do not begin by adding a shader mode. A layer is complete only when its data meaning, generation, sampling, inspection, presentation, and verification agree.

