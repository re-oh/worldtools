# WorldTools Crate Map

This is a source-oriented index for the WorldTools workspace. Use it to answer
"where is that implemented?" without searching the entire repository. Links
are relative and work from an offline checkout.

For behavior and data-flow explanations, read the
[architecture guide](architecture.md).

## Workspace at a glance

| Member | Kind | Depends on WorldTools crates | Primary responsibility |
| --- | --- | --- | --- |
| [`apps/worldtools`](../../apps/worldtools) | Binary | all product crates | Desktop composition, bridges, workflows |
| [`crates/worldtools_world`](../../crates/worldtools_world) | Library | none | Seeds, spherical geometry, base terrain, cube tiles |
| [`crates/worldtools_simulation`](../../crates/worldtools_simulation) | Library | world | Coupled immutable world-history snapshot |
| [`crates/worldtools_render`](../../crates/worldtools_render) | Library/plugin | world, simulation | Map navigation, page streaming, shaders, GPU surfaces |
| [`crates/worldtools_ui`](../../crates/worldtools_ui) | Library/plugin | none | egui editor model and shell |
| [`crates/worldtools_analysis`](../../crates/worldtools_analysis) | Library | world, simulation | Pure numerical and continuity audits |
| [`tools/worldtools_lab`](../../tools/worldtools_lab) | Binary | world, simulation, analysis | Headless generation, images, JSON reports |
| [`xtask`](../../xtask) | Binary | none | Doctor, repro/capture, checks, debugger scripts |

The root [`Cargo.toml`](../../Cargo.toml) pins shared versions and lints. Unsafe
Rust is forbidden workspace-wide and ignored `Result`-like values are denied by
`unused_must_use`.

## Dependency graph

```text
                   +------------------+
                   | worldtools_world |
                   +--------+---------+
                            |
                   +--------v--------------+
                   | worldtools_simulation |
                   +----+-------------+----+
                        |             |
             +----------v--+      +---v----------------+
             | analysis    |      | render             |
             +------+------+
                    |                 |
                    +-------+---------+
                            |
                     +------v------+
                     | desktop app |----> ui
                     +-------------+

lab --> world + simulation + analysis
xtask is independent
```

This is conceptual rather than a literal edge-complete Cargo graph: the app
directly imports all five product libraries.

## `worldtools_world`

Entry point: [`src/lib.rs`](../../crates/worldtools_world/src/lib.rs)

Public API: `CubeFace`, `TileId`, `GeoPoint`, `WorldSeed`, `SeedKey`,
`TerrainSettings`, `TerrainGenerator`, `TerrainTile`, `TerrainTileStats`,
`TerrainTileCache`, and `TileCacheKey`.

This crate is Bevy-free and renderer-free. It is the right home for portable
world identity, mathematical geography, and base terrain that multiple clients
can share.

| File | Key contents | Read it when... |
| --- | --- | --- |
| [`lib.rs`](../../crates/worldtools_world/src/lib.rs) | Modules, re-exports, prelude | You need the public boundary |
| [`seed.rs`](../../crates/worldtools_world/src/seed.rs) | `WorldSeed`, domain-separated `SeedKey` | Adding deterministic randomness |
| [`geo.rs`](../../crates/worldtools_world/src/geo.rs) | `GeoPoint`, sphere/cube conversion, angular distance | Working with coordinates or seams |
| [`tile.rs`](../../crates/worldtools_world/src/tile.rs) | `CubeFace`, `TileId`, quadtree/sample geometry, tile constants | Changing cube-sphere tiling or LOD identity |
| [`terrain.rs`](../../crates/worldtools_world/src/terrain.rs) | Settings, continuous noise source, `TerrainTile` | Changing base elevation generation |
| [`cache.rs`](../../crates/worldtools_world/src/cache.rs) | Weighted Moka terrain tile cache | Caching cube-sphere terrain tiles |
| [`tests/world_properties.rs`](../../crates/worldtools_world/tests/world_properties.rs) | Property tests across seeds/faces/levels | Checking global determinism and continuity invariants |

### Important distinctions

- `world::TileId` is a cube-sphere quadtree id. It is not the renderer's
  equirectangular `render::MapTileId`.
- `TerrainGenerator` is the continuous procedural baseline. It is not the
  complete evolved world.
- `TerrainTile` stores an apron and statistics. Consumers should use its access
  methods rather than duplicate storage-index arithmetic.
- `TerrainSettings::fingerprint` is cache identity; `WorldSeed` domain keys are
  random-process identity.

## `worldtools_simulation`

Entry point: [`src/lib.rs`](../../crates/worldtools_simulation/src/lib.rs)

Public API: `AtlasGrid`, `SimulationSettings`, `WorldSnapshot`, `WorldDataLayer`,
`WorldSample`, layer-specific samples, and categorical enums such as `Biome`,
`KoppenZone`, `SoilKind`, `Lithology`, and `ResourceDeposit`.

This crate owns coupled world history. Its output is renderer-neutral and
immutable.

| File | Key contents | Read it when... |
| --- | --- | --- |
| [`lib.rs`](../../crates/worldtools_simulation/src/lib.rs) | Public modules and prelude | Finding supported contracts |
| [`settings.rs`](../../crates/worldtools_simulation/src/settings.rs) | `SimulationSettings`, defaults, sanitization | Adding or bounding generation controls |
| [`grid.rs`](../../crates/worldtools_simulation/src/grid.rs) | `AtlasGrid`, wrap/index/sample/slope operations | Working with global atlas coordinates |
| [`layers.rs`](../../crates/worldtools_simulation/src/layers.rs) | Public layer enums and sample records | Adding a displayed or inspected world field |
| [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs) | Stage composition, immutable channel storage, sampling, fingerprint | Understanding complete world generation |
| [`random.rs`](../../crates/worldtools_simulation/src/random.rs) | Deterministic simulation random support | Adding seeded stage variation |

### Stage modules

All are under [`src/stages`](../../crates/worldtools_simulation/src/stages/mod.rs).
Their mutable state structs are internal implementation details assembled by
`WorldSnapshot::generate`.

| Module | Main role |
| --- | --- |
| [`tectonics.rs`](../../crates/worldtools_simulation/src/stages/tectonics.rs) | Plates, crust history, boundaries, uplift and volcanic effects |
| [`climate.rs`](../../crates/worldtools_simulation/src/stages/climate.rs) | Temperature, precipitation, seasonality, winds, aridity |
| [`hydrology.rs`](../../crates/worldtools_simulation/src/stages/hydrology.rs) | Flow routing, runoff, rivers, lakes, erosion, sediment |
| [`glaciation.rs`](../../crates/worldtools_simulation/src/stages/glaciation.rs) | Ice motion and glacial surface effects |
| [`geology.rs`](../../crates/worldtools_simulation/src/stages/geology.rs) | Lithology, age, ash, sediment and weathering |
| [`ecology.rs`](../../crates/worldtools_simulation/src/stages/ecology.rs) | Soil and vegetation/biome classification |
| [`resources.rs`](../../crates/worldtools_simulation/src/stages/resources.rs) | Deposit type, richness, depth and confidence |
| [`multires.rs`](../../crates/worldtools_simulation/src/stages/multires.rs) | Multi-resolution transfer helpers |
| [`math.rs`](../../crates/worldtools_simulation/src/stages/math.rs) | Shared numeric helpers for stages |
| [`mod.rs`](../../crates/worldtools_simulation/src/stages/mod.rs) | Internal stage exports and state flow | Tracing stage ownership |

### Sampling entry points

| Method | Returns | Typical caller |
| --- | --- | --- |
| `sample_elevation` | Detailed procedural baseline plus evolved atlas delta | Render pages, probes |
| `sample_atlas_elevation` | Evolved atlas only | Coarse fallback/analysis |
| `sample_slope` | Stored atlas slope | Analysis or broad display |
| `sample` | Complete `WorldSample` | Inspector and audits |
| `sample_layer` | Four `f32` shader channels | `MapTileData` generation |

## `worldtools_render`

Entry point: [`src/lib.rs`](../../crates/worldtools_render/src/lib.rs)

Public plugin: `WorldToolsRenderPlugin`.

Public integration types include `MapView`, render `MapViewport`,
`MapNavigationSettings`, `MapDisplaySettings`, `MapDisplayMode`, `MapTileId`,
`MapTileStreamer`, `MapTileData`, `VisibleMapTiles`, `TileStreamStats`,
`TileRenderStats`, and `RenderDebugSettings`.

| File | Key contents | Read it when... |
| --- | --- | --- |
| [`lib.rs`](../../crates/worldtools_render/src/lib.rs) | Public re-exports | Finding the app-facing API |
| [`surface.rs`](../../crates/worldtools_render/src/surface.rs) | Root render plugin, camera, legacy full-surface material | Understanding plugin composition/startup |
| [`view.rs`](../../crates/worldtools_render/src/view.rs) | Map center/span, viewport, pan/zoom input | Changing navigation or viewport math |
| [`projection.rs`](../../crates/worldtools_render/src/projection.rs) | `MapTileId`, placement, visible plan/LOD | Changing page selection or dateline behavior |
| [`streaming.rs`](../../crates/worldtools_render/src/streaming.rs) | Snapshot owner, cache, workers, invalidation, stats | Debugging missing/stale pages |
| [`tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs) | Samples one snapshot into elevation/layer images | Changing CPU-to-GPU page payload |
| [`tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs) | GPU cache, entities, ancestor fallback, tile uniforms | Debugging what is actually drawn |
| [`tile_material.rs`](../../crates/worldtools_render/src/tile_material.rs) | Per-page Bevy material and shader registration | Changing page bindings/uniform layout |
| [`display.rs`](../../crates/worldtools_render/src/display.rs) | Modes and display/style/lighting values | Adding presentation controls |
| [`debug.rs`](../../crates/worldtools_render/src/debug.rs) | Render toggles and render counters | Adding visual diagnostics |
| [`blue_noise.rs`](../../crates/worldtools_render/src/blue_noise.rs) | Deterministic noise texture | Understanding shader dithering input |
| [`height_field.rs`](../../crates/worldtools_render/src/height_field.rs) | Legacy full-surface upload contract | Working on the old surface path |
| [`material.rs`](../../crates/worldtools_render/src/material.rs) | Legacy full-surface material | Working on the old shader path |

### Shader files

| File | Used by |
| --- | --- |
| [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl) | Streamed per-page `TerrainTileMaterial` |
| [`worldtools_terrain.wgsl`](../../crates/worldtools_render/src/worldtools_terrain.wgsl) | Legacy `TerrainMaterial` full surface |

### Runtime resources by owner

| Owner | Resources/messages |
| --- | --- |
| `WorldToolsRenderPlugin` | `MapView`, render `MapViewport`, navigation state/settings, display/debug settings, `HeightFieldUpload` |
| `TileStreamingPlugin` | `MapTileStreamer`, `VisibleMapTiles`, `TileStreamStats`, `MapTileInvalidation` |
| `TileSurfacePlugin` | GPU cache, rendered tile map, shared assets, `TileRenderStats` |

The streamer owns the authoritative active snapshot in the desktop runtime.
Rendered entities and GPU assets are derived from it.

## `worldtools_ui`

Entry point: [`src/lib.rs`](../../crates/worldtools_ui/src/lib.rs)

Public plugin: `WorldToolsUiPlugin`.

This crate owns editor semantics and drawing, not application effects. It may
emit `RegenerateWorld` or `DebugCommand`, but app plugins consume them.

### Public model

[`model.rs`](../../crates/worldtools_ui/src/model.rs) re-exports the complete
integration surface.

| File | Important types |
| --- | --- |
| [`model/editor.rs`](../../crates/worldtools_ui/src/model/editor.rs) | `EditorUiState` |
| [`model/tools.rs`](../../crates/worldtools_ui/src/model/tools.rs) | `ActiveTool`, `MapViewMode`, `WorldLayer` |
| [`model/document.rs`](../../crates/worldtools_ui/src/model/document.rs) | `DocumentStatus` |
| [`model/generation.rs`](../../crates/worldtools_ui/src/model/generation.rs) | `PipelineStage`, `GenerationActivity`, `GenerationStatus`, `WorldGenerationDraft` |
| [`model/commands.rs`](../../crates/worldtools_ui/src/model/commands.rs) | `RegenerateWorld` |
| [`model/viewport.rs`](../../crates/worldtools_ui/src/model/viewport.rs) | UI `MapViewport`, `ViewportRect`, `MapReadout` |
| [`model/presentation.rs`](../../crates/worldtools_ui/src/model/presentation.rs) | `LayerVisualStyle`, `MapPresentationSettings` |
| [`model/probe.rs`](../../crates/worldtools_ui/src/model/probe.rs) | `MapProbe`, `LayerProbe`, `TerrainProbe`, `ProbeReading` |
| [`model/debug.rs`](../../crates/worldtools_ui/src/model/debug.rs) | Debug state, commands, events, telemetry, layer capabilities |
| [`model/analysis.rs`](../../crates/worldtools_ui/src/model/analysis.rs) | `AnalysisStatus`, `AnalysisIssue`, severity |

There are two `MapViewport` types by design. The UI version captures egui
logical/physical rectangles. The render version uses Bevy vectors plus scale
and input policy. The app's viewport bridge translates between them.

### Shell

The composition function is
[`shell::draw_editor_shell`](../../crates/worldtools_ui/src/shell/mod.rs).

| File | Visible area |
| --- | --- |
| [`menu_bar.rs`](../../crates/worldtools_ui/src/shell/menu_bar.rs) | Top menu/document identity/debug access |
| [`tool_rail.rs`](../../crates/worldtools_ui/src/shell/tool_rail.rs) | Navigate and Inspect tools |
| [`explorer.rs`](../../crates/worldtools_ui/src/shell/explorer.rs) | Layer and map-view selection |
| [`inspector.rs`](../../crates/worldtools_ui/src/shell/inspector.rs) | Generation and selected-point properties |
| [`viewport.rs`](../../crates/worldtools_ui/src/shell/viewport.rs) | Measures map region and blocking state |
| [`legend.rs`](../../crates/worldtools_ui/src/shell/legend.rs) | Active-layer legend |
| [`bottom_drawer.rs`](../../crates/worldtools_ui/src/shell/bottom_drawer.rs) | Analysis results |
| [`status_bar.rs`](../../crates/worldtools_ui/src/shell/status_bar.rs) | Generation, cursor, scale, LOD |
| [`debug_window.rs`](../../crates/worldtools_ui/src/shell/debug_window.rs) | Debug controls, telemetry, events |

[`style.rs`](../../crates/worldtools_ui/src/style.rs) centralizes egui visuals;
[`widgets.rs`](../../crates/worldtools_ui/src/widgets.rs) centralizes compact
editor widgets.

## Desktop app: `worldtools`

Entry point: [`src/main.rs`](../../apps/worldtools/src/main.rs)

The app is the composition root. It is the correct owner for code that must
understand contracts from two or more otherwise independent crates.

| File | Responsibility | Key contracts |
| --- | --- | --- |
| [`main.rs`](../../apps/worldtools/src/main.rs) | Process setup, Bevy plugins/window | Adds render, UI, generation, interaction, bridge, debug |
| [`generation.rs`](../../apps/worldtools/src/generation.rs) | Complete snapshot regeneration | `RegenerateWorld` -> async `WorldSnapshot` -> streamer replacement |
| [`layers.rs`](../../apps/worldtools/src/layers.rs) | UI/simulation layer mapping | `WorldLayer` -> `WorldDataLayer` |
| [`viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs) | UI-to-render resource synchronization | Viewport, tools, layer, styles, modes |
| [`interaction.rs`](../../apps/worldtools/src/interaction.rs) | Capabilities, cursor/readout | Render stats/view -> UI `MapReadout` |
| [`interaction/probe.rs`](../../apps/worldtools/src/interaction/probe.rs) | Inspect-click sampling | Streamer snapshot -> UI `MapProbe` |
| [`interaction/probe/format.rs`](../../apps/worldtools/src/interaction/probe/format.rs) | Layer sample formatting | `WorldSample` -> human-readable readings |
| [`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs) | Tracing files/events and panic reports | Process setup -> Bevy diagnostic resources |
| [`live_remote.rs`](../../apps/worldtools/src/live_remote.rs) | Optional Bevy Remote | `live-debug` feature plus runtime opt-in |

### Debug tools

Entry point: [`debug_tools/mod.rs`](../../apps/worldtools/src/debug_tools/mod.rs)

| File | Responsibility |
| --- | --- |
| [`commands.rs`](../../apps/worldtools/src/debug_tools/commands.rs) | Applies UI debug commands to render/app state |
| [`telemetry.rs`](../../apps/worldtools/src/debug_tools/telemetry.rs) | Drains trace events and publishes frame/stream/render metrics |
| [`snapshot.rs`](../../apps/worldtools/src/debug_tools/snapshot.rs) | Writes diagnostic state snapshots |
| [`audit.rs`](../../apps/worldtools/src/debug_tools/audit.rs) | Starts and receives terrain audits |
| [`io.rs`](../../apps/worldtools/src/debug_tools/io.rs) | Diagnostic file helpers |

The app debug plugin chains these systems so commands, captures, and audit
results observe a predictable order within its pipeline.

## `worldtools_analysis`

Entry point: [`src/lib.rs`](../../crates/worldtools_analysis/src/lib.rs)

This crate is pure, reusable analysis. It accepts domain values and returns
serializable/report-friendly records; it owns no ECS schedules or output paths.

| File | Public types/functions | What it measures |
| --- | --- | --- |
| [`distribution.rs`](../../crates/worldtools_analysis/src/distribution.rs) | `Distribution`, `Quantiles` | Finite counts, range, moments, quantiles, normalized entropy |
| [`terrain.rs`](../../crates/worldtools_analysis/src/terrain.rs) | `TerrainAudit`, `audit_terrain*` | Elevation, land fraction, slope, ruggedness, extrema |
| [`aggregate.rs`](../../crates/worldtools_analysis/src/aggregate.rs) | `TerrainAggregate`, `aggregate_terrain` | Correctly weighted multi-tile summaries |
| [`seam.rs`](../../crates/worldtools_analysis/src/seam.rs) | `TileEdge`, `SeamAudit`, seam audit functions | Error along coincident cube-sphere edges |
| [`lod.rs`](../../crates/worldtools_analysis/src/lod.rs) | `LodAudit`, `audit_child_consistency` | Error at coincident parent/child samples |
| [`simulation.rs`](../../crates/worldtools_analysis/src/simulation.rs) | `WorldSimulationAudit`, coherence records | Finiteness and cross-process plausibility |

Analysis reports make their weighting explicit. Terrain per-tile distributions
are vertex-sample weighted; whole-tile aggregation uses recorded sample counts.
Do not silently treat those as exact sphere-area weighting.

## `worldtools_lab`

Entry point: [`src/main.rs`](../../tools/worldtools_lab/src/main.rs)

This headless binary is the easiest way to study deterministic domain behavior
without Bevy, windowing, or GPU state.

| File | Responsibility |
| --- | --- |
| [`args.rs`](../../tools/worldtools_lab/src/args.rs) | Clap commands and arguments |
| [`preview.rs`](../../tools/worldtools_lab/src/preview.rs) | Overview and individual tile PNG rendering |
| [`verify.rs`](../../tools/worldtools_lab/src/verify.rs) | Exhaustive level verification |
| [`sweep.rs`](../../tools/worldtools_lab/src/sweep.rs) | Cross-seed aggregate comparison |
| [`world.rs`](../../tools/worldtools_lab/src/world.rs) | Coupled snapshot generation and audit |
| [`tile_set.rs`](../../tools/worldtools_lab/src/tile_set.rs) | Tile set generation/traversal helpers |
| [`cube_edges.rs`](../../tools/worldtools_lab/src/cube_edges.rs) | Cube face edge relationships |
| [`continuity.rs`](../../tools/worldtools_lab/src/continuity.rs) | Combines seam and LOD errors |
| [`report.rs`](../../tools/worldtools_lab/src/report.rs) | JSON to stdout or file |

Command routing is deliberately thin:

```text
Cli::parse
  +-- overview --> preview::overview
  +-- tile -----> preview::tile
  +-- verify ---> verify::verify
  +-- sweep ----> sweep::sweep
  +-- world ----> world::world
```

## `xtask`

Entry point: [`src/main.rs`](../../xtask/src/main.rs)

`xtask` is an operational harness, not a domain library. It remains useful when
the application itself does not build or start.

| File | Responsibility |
| --- | --- |
| [`cli.rs`](../../xtask/src/cli.rs) | `doctor`, `repro`, `capture`, `check`, `debug-script` arguments |
| [`doctor.rs`](../../xtask/src/doctor.rs) | Platform, repository, toolchain, debugger capability JSON |
| [`case.rs`](../../xtask/src/case.rs) | Deterministic TOML debug-case schema and validation |
| [`reproduce.rs`](../../xtask/src/reproduce.rs) | Repeated controlled execution and artifact recording |
| [`process.rs`](../../xtask/src/process.rs) | Timeout-aware child process capture/inherited runs |
| [`artifact.rs`](../../xtask/src/artifact.rs) | Run directories and atomic-ish report writing helpers |
| [`check.rs`](../../xtask/src/check.rs) | Sequential quick/full verification lanes |
| [`debug_script.rs`](../../xtask/src/debug_script.rs) | LLDB/CDB noninteractive script generation |
| [`workspace.rs`](../../xtask/src/workspace.rs) | Root and debug-case path resolution |

Runtime-generated repro and capture output belongs under `.debug`, while app
diagnostic output belongs under `.runtime`; neither is product source.

## Cross-crate contracts

These are the most useful boundaries to recognize during code review.

| Producer | Contract | Consumer |
| --- | --- | --- |
| world | `WorldSeed`, `TerrainSettings`, `GeoPoint` | simulation, render, analysis, lab |
| simulation | `Arc<WorldSnapshot>`, `WorldDataLayer`, `WorldSample` | render, analysis, app, lab |
| UI | `RegenerateWorld` | app generation plugin |
| UI | `DebugCommand` | app debug plugin |
| UI | viewport/editor/presentation resources | app viewport bridge |
| render | `MapView`, `TileStreamStats`, snapshot sampling | app interaction/telemetry |
| app bridge | render viewport/navigation/display resources | render systems |
| app generation | replacement `Arc<WorldSnapshot>` | `MapTileStreamer` |
| analysis | audit records | app debug tools and lab reports |

## Where to make common changes

| Change | Start here | Then inspect |
| --- | --- | --- |
| New terrain noise parameter | `world/terrain.rs` | settings fingerprint, lab args/reports, tests |
| New global process | `simulation/stages` | snapshot composition/storage, analysis |
| New inspectable layer | `simulation/layers.rs` | snapshot sampling, UI model, app mapping/format, shader |
| Different LOD selection | `render/projection.rs` | streaming stats, tile placement tests |
| Streaming bug | `render/streaming.rs` | projection plan, tile surface fallback, telemetry |
| Wrong colors/relief | render display/WGSL | UI presentation and viewport bridge |
| Editor layout | `ui/shell` | UI model; avoid render imports |
| Regeneration lifecycle | `app/generation.rs` | UI status/message, streamer replacement |
| Probe mismatch | `app/interaction/probe.rs` | pointer transform, snapshot sampling, formatting |
| Numerical quality metric | `analysis` | lab or debug orchestration |
| Reproduction/capture behavior | `xtask` | `.debug/cases` and generated artifacts |

## Names that are easy to confuse

| Name | Meaning |
| --- | --- |
| `TileId` | Cube-sphere domain tile from `worldtools_world` |
| `MapTileId` | Equirectangular streamed render page |
| UI `MapViewport` | egui rectangle and input-blocking measurement |
| Render `MapViewport` | Bevy-space bounds, scale, input policy |
| `TerrainSettings` | Base procedural surface parameters |
| `SimulationSettings` | Global atlas/process parameters |
| `TerrainTile` | Cube-sphere elevation tile from base generator |
| `MapTileData` | Equirectangular render page sampled from a snapshot |
| `WorldLayer` | UI selection vocabulary |
| `WorldDataLayer` | Simulation/render sampling vocabulary |
| `revision` | Stable snapshot fingerprint prefix or per-page invalidation counter, depending on context |
| `world_epoch` | Runtime generation number for complete snapshot replacement |

## Minimal reading order by concern

### Understand world generation

1. [`world/seed.rs`](../../crates/worldtools_world/src/seed.rs)
2. [`world/terrain.rs`](../../crates/worldtools_world/src/terrain.rs)
3. [`simulation/settings.rs`](../../crates/worldtools_simulation/src/settings.rs)
4. [`simulation/snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs)
5. The relevant file under [`simulation/stages`](../../crates/worldtools_simulation/src/stages/mod.rs)

### Understand one rendered frame

1. [`app/viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs)
2. [`render/view.rs`](../../crates/worldtools_render/src/view.rs)
3. [`render/projection.rs`](../../crates/worldtools_render/src/projection.rs)
4. [`render/streaming.rs`](../../crates/worldtools_render/src/streaming.rs)
5. [`render/tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs)
6. [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl)

### Understand one user action

1. Find the control under [`ui/shell`](../../crates/worldtools_ui/src/shell/mod.rs).
2. Find its resource/message under [`ui/model`](../../crates/worldtools_ui/src/model.rs).
3. Find the app system that consumes or bridges it.
4. Follow the called render/simulation public method into its owning crate.

### Understand a diagnostic artifact

1. [`app/diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs) for logs/panics.
2. [`app/debug_tools`](../../apps/worldtools/src/debug_tools/mod.rs) for in-app
   telemetry, snapshots, and audits.
3. [`analysis`](../../crates/worldtools_analysis/src/lib.rs) for metric meaning.
4. [`xtask/reproduce.rs`](../../xtask/src/reproduce.rs) for deterministic run
   metadata, stdout, stderr, and environment evidence.
