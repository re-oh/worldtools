# Rendering, UI, and Runtime Integration

This chapter explains how a generated `WorldSnapshot` becomes the interactive map shown by WorldTools. It is written for offline use: every link points to source in this checkout.

## The ownership boundary

The application is assembled in [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs):

```text
worldtools_simulation                 immutable WorldSnapshot
          |
          v
worldtools_render                     sampling, tiles, GPU assets, navigation
          ^
          | translated resources
apps/worldtools/viewport_bridge.rs    editor intent -> renderer settings
          ^
          |
worldtools_ui                         egui layout, controls, public UI state
```

The split is deliberate:

- [`worldtools_ui`](../../crates/worldtools_ui/src/lib.rs) does not know how terrain is stored or rendered. It publishes resources and messages describing user intent.
- [`viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs) is the adapter. It copies the measured egui viewport, active tool, active data layer, view mode, and presentation controls into renderer resources.
- [`worldtools_render`](../../crates/worldtools_render/src/lib.rs) owns map navigation, projection, tile planning, asynchronous page generation, caches, GPU images, materials, and shaders.
- The app plugins for [generation](../../apps/worldtools/src/generation.rs), [interaction](../../apps/worldtools/src/interaction.rs), and [debug tools](../../apps/worldtools/src/debug_tools/mod.rs) connect the remaining workflows.

When adding a control, put editor-facing state in the UI model, translation in the app bridge, and rendering behavior in the renderer. This keeps the reusable UI crate independent of the renderer.

## Frame-level data flow

The important update path is:

```text
egui shell measures central viewport
  -> UI MapViewport (points + physical pixels + input ownership)
  -> ViewportBridge converts to Bevy logical window coordinates
  -> renderer MapViewport
  -> tile planner selects a LOD and visible placements
  -> streamer requests missing pages from AsyncComputeTaskPool
  -> completed CPU pages enter the resident cache
  -> tile surface uploads R32Float + Rgba32Float images
  -> TerrainTileMaterial draws one rectangle per placement
  -> embedded WGSL presents the selected data layer
```

The systems are intentionally incremental. A view change can immediately draw an ancestor page while the exact page is being generated. A layer change invalidates old pages, but revision and layer checks prevent late asynchronous results from entering the current cache.

## View coordinates and navigation

[`MapView`](../../crates/worldtools_render/src/view.rs) stores:

- `center.x`: normalized longitude. It is an unwrapped `f64`, so repeated travel across the date line does not lose world-wrap continuity.
- `center.y`: normalized north-to-south position, clamped so the vertical view does not leave the poles.
- `vertical_span`: visible fraction of the pole-to-pole height, clamped from `1.0` down to `1 / 32768`.

Horizontal span is derived from vertical span and viewport aspect ratio. The extra factor of `0.5` reflects the 2:1 equirectangular world. Zoom is anchored under the cursor. Middle-button pan is always available; primary-button pan is enabled only when the UI's active tool is Navigate.

The UI and renderer use different coordinate spaces:

```text
egui points --(egui pixels_per_point)--> physical pixels
physical pixels --(window scale factor)--> Bevy logical window coordinates
```

[`crates/worldtools_ui/src/shell/viewport.rs`](../../crates/worldtools_ui/src/shell/viewport.rs) records both point-space and pixel-aligned rectangles. [`MapViewport::window_logical`](../../crates/worldtools_ui/src/model/viewport.rs) accounts for UI zoom and native display scaling. The bridge also copies `input_blocked`; map gestures are suppressed while egui owns the pointer or the pointer is outside the map.

If the map is offset, scaled incorrectly, or reacts through a panel, inspect this coordinate conversion before changing projection math.

## Projection, LOD, and world wrapping

The tile pyramid is defined in [`projection.rs`](../../crates/worldtools_render/src/projection.rs):

- Every page covers `256 x 256` logical cells.
- Level `L` has `2^(L+1)` columns and `2^L` rows, preserving the 2:1 world.
- The maximum level is 17.
- Tile planning includes a one-page prefetch margin.
- A plan is reduced to a coarser level when it would exceed 64 visible placements.

The desired level is chosen from physical pixel density: the planner seeks a page density appropriate for the visible vertical world span. Using physical rather than logical size makes high-DPI displays request sufficient source resolution.

Longitude uses two identities:

- `MapTileId.x` is canonical and wraps with modulo into the finite tile pyramid. It is used as the cache key.
- `MapTilePlacement.unwrapped_x` records which visual copy of that canonical page is on screen. It is used for stable placement across the longitude seam.

Latitude does not wrap. Y planning is clamped to the poles. Rendering converts only camera-relative values to `f32`; the persistent view center remains `f64` so small pans still work after many wraps.

## Tile data and seam handling

[`MapTileData`](../../crates/worldtools_render/src/tile_data.rs) is a CPU page sampled from one immutable snapshot and one `WorldDataLayer`. Each page stores:

- elevation as `R32Float`;
- four layer channels as `Rgba32Float`;
- the source tile ID, active layer, and elevation range.

Although the logical page is 257 samples across including both cell boundaries, storage adds a four-sample apron on every side. The resulting image is `265 x 265`, or 70,225 samples. Longitude samples naturally continue through world wrap; latitude samples beyond the world edge are clamped.

The apron allows bicubic interpolation, terrain derivatives, broad normals, horizon probes, and category-boundary tests to read across a page edge. Adjacent pages sample identical global coordinates at their shared boundary. That combination is the core seam strategy: deterministic shared samples plus enough neighbor context for the shader footprint.

At level 0, elevation may use the snapshot atlas as a fast coarse fallback. Finer elevation tiles use the full elevation sampler. Non-elevation pages always carry base elevation in the elevation texture and the selected layer in the four-channel texture.

## Streaming and cache lifecycle

[`MapTileStreamer`](../../crates/worldtools_render/src/streaming.rs) owns an immutable `Arc<WorldSnapshot>`, the active layer, and the CPU page lifecycle:

```text
VisibleMapTiles
  -> priority list: roots, exact desired pages, intermediate ancestors
  -> at most 8 jobs in flight
  -> bounded result channel
  -> at most 8 results accepted per frame
  -> Moka resident cache, capacity 128 pages
```

Roots are requested early so the renderer can cover the map before exact pages finish. `best_available` walks from a desired page to its parents. This produces three useful render states:

- **Exact**: the desired page is resident.
- **Fallback**: a resident ancestor is cropped and scaled for the desired placement.
- **Stale**: the placement retains its previous GPU page while replacement work is pending.

If no exact, fallback, or retained page exists, the placement is **missing** and no tile entity is drawn for it.

Every tile has a revision. Invalidation removes its resident entry and increments the revision; a completed job is accepted only if both its revision and its layer still match. Changing the active layer invalidates resident and in-flight IDs. Replacing the world snapshot increments `world_epoch`, creates fresh channels and caches, and causes the GPU tile surface to discard all entities and images from the old epoch.

Freezing streaming stops new requests but lets in-flight work finish. It is a diagnostic scheduler control, not a snapshot of all tile state.

## CPU pages to GPU pages

[`tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs) maintains two renderer-side collections:

- `GpuTileCache`: source tile ID to elevation image, layer image, and the exact `Arc<MapTileData>` that produced them.
- `RenderedTiles`: visual placement to Bevy entity, material handle, source ID, and source layer.

A source page can serve several desired child placements. `sample_rect` tells the shader which subrectangle of an ancestor image belongs to a fallback placement. The placement transform uses `unwrapped_x`, the camera-relative offset, and the measured viewport to position and size a unit rectangle.

The GPU cache replaces images when the underlying `Arc` changes and drops images no longer present in the CPU cache. `TileRenderStats` counts rendered, exact, fallback, stale, missing, and GPU-resident pages each frame.

[`surface.rs`](../../crates/worldtools_render/src/surface.rs) also retains the older full-viewport `TerrainSurface`/`HeightFieldUpload` path. It registers the base terrain material and sits behind tile entities. The streamed tile surface is the main world-snapshot presentation path; do not confuse legacy height-field upload state with streamed page residency.

## Materials and shaders

Both shaders are embedded in the binary through Bevy's asset macros, so runtime display does not depend on external shader files:

- [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl) is the streamed-page shader used by `TerrainTileMaterial`.
- [`worldtools_terrain.wgsl`](../../crates/worldtools_render/src/worldtools_terrain.wgsl) is the older full-surface shader.

[`TerrainTileMaterialParams`](../../crates/worldtools_render/src/tile_material.rs) groups the per-placement uniforms:

| Field | Purpose |
|---|---|
| `sample_rect` | Source image origin and span, including fallback-page cropping. |
| `metrics` | Metres per sample, dither amplitude, and latitude. |
| `debug` | Overlay flags, border width, desired LOD, and source LOD. |
| `display` | Display mode, sea level, contour interval, relief strength. |
| `style` | Shadow, fine detail, category boundary, layer opacity. |
| `lighting` | Sun azimuth/elevation and ambient occlusion. |
| `world` | Canonical normalized origin/span for stable procedural markers. |

The shader samples elevation bicubically and computes the elevation derivatives from the same 4x4 footprint. Continuous layer values use bounded bicubic interpolation to prevent overshoot. Category IDs use nearest sampling so a boundary never becomes a fractional category.

Presentation is 2.5D rather than mesh displacement. The shader combines fine and broad terrain normals, local concavity, a short horizon test, configurable sun direction, shadows, and ambient occlusion. A small tiled blue-noise value reduces visible quantization.

## Display and presentation modes

[`MapDisplaySettings`](../../crates/worldtools_render/src/display.rs) is uniform-only state. It does not generate new data. Values are sanitized and clamped before reaching the GPU.

The elevation dataset supports Physical, Elevation, Relief, Slope, and Contours. The other modes present native simulation channels for Tectonics, Hydrology, Climate, Soil, Vegetation, Geology, and Resources. The bridge chooses the shader mode from `EditorUiState.active_layer` and the elevation `MapViewMode`.

[`MapPresentationSettings`](../../crates/worldtools_ui/src/model/presentation.rs) stores per-layer opacity, relief, shadows, detail, boundary strength, and label visibility, plus global lighting. The bridge copies the active layer's style into renderer uniforms every frame. Categorical borders apply to tectonics, soil, vegetation, and geology; continuous layers retain smooth interpolation. Layer modes blend over the physical terrain using the configured opacity.

When a stale page belongs to the previous active layer, the tile is temporarily presented as Physical rather than interpreting old channel values with the new layer's palette.

## Egui shell state and messages

[`WorldToolsUiPlugin`](../../crates/worldtools_ui/src/lib.rs) installs egui and initializes the editor contract. The main groups are:

- `EditorUiState`: selected tool, layer, and elevation view mode.
- `WorldGenerationDraft`, `GenerationStatus`, `DocumentStatus`: generation input and lifecycle.
- `MapViewport`, `MapReadout`, `MapProbe`: measured map area and inspection output.
- `MapPresentationSettings`: visual intent.
- `DebugUiState`, `DebugTelemetry`, `DebugEventLog`: diagnostics controls and observations.
- `RegenerateWorld` and `DebugCommand`: requests consumed by app integration plugins.

The shell is drawn in `EguiPrimaryContextPass` by [`draw_editor_shell`](../../crates/worldtools_ui/src/shell/mod.rs). Menu, explorer, tool rail, inspector, bottom drawer, status bar, central viewport, and diagnostics window all mutate these public resources or emit messages. They do not call renderer internals.

F12 toggles diagnostics. H selects Navigate and I selects Inspect unless egui wants keyboard input. The viewport marks input blocked while an overlay or control owns the pointer, preventing both egui and the map from acting on one gesture.

## Integration checklist

When adding a render-facing UI feature:

1. Add editor intent to an appropriate UI model under [`crates/worldtools_ui/src/model`](../../crates/worldtools_ui/src/model.rs).
2. Draw the control in the relevant shell module and keep it independent of renderer types.
3. Translate the state in [`viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs), or emit a message for a command-like action.
4. Add or update a renderer resource and consume it in a focused system.
5. Put data generation in simulation/world code; keep presentation-only changes in uniforms or shaders.
6. Expose diagnostic state when the feature has asynchronous, cached, or GPU-visible behavior.
7. Verify standard DPI and high-DPI viewport placement, longitude wrapping, and input blocking.

For diagnosis and retained evidence, continue with [Debugging and Diagnostics](debugging.md).
