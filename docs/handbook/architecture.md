# WorldTools Architecture

This chapter explains how WorldTools is put together, which part owns each kind
of state, and how data moves from a seed to pixels and editor readouts. Every
link points to source in this checkout, so the chapter works offline.

For a faster file lookup, see the [crate map](crate-map.md). For a guided first
read, see the [reading tours](reading-tours.md).

## The system in one picture

WorldTools separates deterministic world computation from the Bevy application
that displays it:

```text
                         engine-independent
                    +-------------------------+
                    | worldtools_world        |
 seed + settings -->| geometry, base terrain  |
                    +------------+------------+
                                 |
                                 v
                    +-------------------------+
                    | worldtools_simulation   |
                    | immutable WorldSnapshot |
                    +----+---------------+----+
                         |               |
              sample/audit               | sample visible pages
                         |               |
                         v               v
              +----------------+  +-------------------+
              | analysis + lab |  | worldtools_render |
              +----------------+  | streaming + GPU   |
                                  +---------+---------+
                                            |
                                  Bevy resources/messages
                                            |
                +---------------------------+------------------+
                | apps/worldtools: composition and bridges     |
                | generation, interaction, diagnostics, debug  |
                +---------------------------+------------------+
                                            |
                                            v
                                  +-------------------+
                                  | worldtools_ui     |
                                  | editor shell/model|
                                  +-------------------+
```

The central contract is [`WorldSnapshot`](../../crates/worldtools_simulation/src/snapshot.rs).
It is an immutable, cheap-to-clone collection of shared atlas channels. Both the
renderer and analysis code sample this same object. The UI does not know its
layout, and the simulation does not know Bevy, egui, shaders, or windows.

## Architectural rules

The code follows five important rules.

1. **Explicit inputs determine generated data.** A world is a function of
   `WorldSeed`, `TerrainSettings`, and `SimulationSettings`. Domain-separated
   seed derivation avoids randomized hashes and accidental coupling.
2. **The completed snapshot is immutable.** Generation builds mutable stage
   state internally, then freezes channels behind `Arc<[T]>` storage. Readers
   cannot observe a half-generated world.
3. **Expensive derived pages are disposable.** Render tiles, CPU caches, GPU
   images, and entities can all be rebuilt from the snapshot. They are not the
   source of truth.
4. **Crates own mechanisms; the app owns policy between crates.** For example,
   the UI defines `RegenerateWorld`, simulation defines generation, render owns
   the active snapshot, and the app connects those contracts.
5. **Stale asynchronous results identify themselves.** Layer identity, tile
   revision, and world epoch stop work from an old view or world entering the
   active caches.

## Workspace dependency direction

The workspace members are declared in the root [`Cargo.toml`](../../Cargo.toml).
The important dependency direction is:

```text
worldtools_world
      ^
      |
worldtools_simulation
      ^             ^
      |             |
worldtools_render   worldtools_analysis
      ^             ^
      |             |
      +------ worldtools app ------> worldtools_ui
                    ^
                    |
             Bevy composition root

worldtools_lab --> world + simulation + analysis
xtask         --> no WorldTools runtime crate
```

`worldtools_world`, `worldtools_simulation`, and `worldtools_analysis` are
engine-independent. `worldtools_render` and `worldtools_ui` depend on Bevy, but
the UI deliberately has no dependency on world storage, simulation, or render.
The desktop app is the only member that depends on every product crate.

## Runtime composition

The desktop entry point is [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs).
It has three responsibilities:

1. Install process diagnostics before Bevy installs tracing.
2. Configure Bevy's default plugins and primary window.
3. Add the WorldTools plugins that cooperate through ECS resources and messages.

The plugin set is:

| Plugin | Owner | Purpose |
| --- | --- | --- |
| `WorldToolsRenderPlugin` | render crate | Map view, page streaming, materials, GPU tile surfaces |
| `WorldToolsUiPlugin` | UI crate | egui shell and public editor resources/messages |
| `WorldGenerationPlugin` | app | Background snapshot replacement |
| `WorldInteractionPlugin` | app | Pointer-to-world inspection and status readout |
| `ViewportBridgePlugin` | app | Copies UI state into renderer resources |
| `WorldToolsDebugPlugin` | app | Telemetry, commands, snapshots, and audits |
| `WorldToolsRemoteDebugPlugin` | app, optional | Loopback Bevy Remote when compiled and enabled |

The optional remote plugin is feature-gated by `live-debug`; its implementation
is in [`live_remote.rs`](../../apps/worldtools/src/live_remote.rs). Normal builds
do not enable it.

### Initialization sequence

The significant startup order is:

```text
process starts
  |
  +-- Diagnostics::install
  |     create output directory, event channel, panic hook
  |
  +-- construct Bevy App
  |     insert diagnostic resources and ClearColor
  |
  +-- DefaultPlugins
  |     tracing/logging and primary 1600 x 900 window
  |
  +-- add WorldTools plugins
  |     resources and messages are registered during plugin build
  |
  +-- app.run
        Startup systems create cameras/assets and publish capabilities
        Update and EguiPrimaryContextPass systems begin running
```

One detail matters when diagnosing slow startup: `TileStreamingPlugin` calls
`init_resource::<MapTileStreamer>()`. The streamer's `Default` implementation
constructs seed 1 with default terrain and simulation settings, and
[`MapTileStreamer::new`](../../crates/worldtools_render/src/streaming.rs) calls
`WorldSnapshot::generate` synchronously. Later user-requested regeneration is
asynchronous, but the initial world is built while resources initialize.

The render plugin also creates two surfaces. The legacy full-window
[`TerrainSurface`](../../crates/worldtools_render/src/surface.rs) starts with a
small flat height image. The actual streamed map is assembled by
[`TileSurfacePlugin`](../../crates/worldtools_render/src/tile_surface.rs) from
per-page entities and materials.

## World foundation: geometry, seeds, and base terrain

[`worldtools_world`](../../crates/worldtools_world/src/lib.rs) is the lowest
domain layer. It owns representations that must stay stable regardless of the
application or renderer.

### Coordinates

[`GeoPoint`](../../crates/worldtools_world/src/geo.rs) stores latitude and
longitude in radians and converts to or from unit sphere directions. The same
module defines cube-face UV conversion and angular distance. Generation samples
a continuous 3D direction, avoiding a longitude discontinuity in the underlying
noise.

[`CubeFace` and `TileId`](../../crates/worldtools_world/src/tile.rs) define a
six-face cube-sphere quadtree. A `TileId` contains face, level, x, and y. It can
find parents and children, map sample coordinates to unit directions, and
compute a conservative spherical cap. Important fixed tile constants are also
owned here:

```text
interior cells       TILE_CELLS
interior vertices    TILE_SAMPLES = TILE_CELLS + 1
processing border    TILE_APRON
stored side length   TILE_STORAGE_SAMPLES
```

The apron lets local processing inspect neighbors. Shared-edge and
parent/child direction calculations are designed to be bit-identical at
coincident samples; the property tests are in
[`world_properties.rs`](../../crates/worldtools_world/tests/world_properties.rs).

### Deterministic seed derivation

[`WorldSeed`](../../crates/worldtools_world/src/seed.rs) is the portable `u64`
root. `WorldSeed::key(domain)` derives a BLAKE3 key using a length-prefixed
domain string. `tile_key(domain, tile)` additionally includes stable tile bytes.
`SeedKey` can supply bytes, a `u64`, or a noise-compatible `i32`.

Use a new descriptive domain whenever a new random process is added. Reusing a
domain couples two systems: changing one process's draw sequence can then alter
another. Never use Rust's default randomized `Hash` as a generation seed.

### Continuous terrain

[`TerrainSettings`, `TerrainGenerator`, and `TerrainTile`](../../crates/worldtools_world/src/terrain.rs)
own the base elevation source. Three independently seeded FastNoiseLite fields
produce continental shape, mountains, and detail. `TerrainGenerator` supports
point sampling and complete cube-sphere tiles. `TerrainSettings::fingerprint`
hashes every setting into stable cache identity.

[`TerrainTileCache`](../../crates/worldtools_world/src/cache.rs) is a weighted,
thread-safe Moka cache keyed by seed, terrain fingerprint, and `TileId`. It is a
generic world-layer facility; the current equirectangular renderer has its own
page cache because its page geometry and layer payload differ.

## Coupled world simulation

[`worldtools_simulation`](../../crates/worldtools_simulation/src/lib.rs) turns a
base terrain source into long-timescale world history. It remains independent
of Bevy and can run in the desktop app, tests, or the headless lab.

### Global atlas

[`AtlasGrid`](../../crates/worldtools_simulation/src/grid.rs) describes the
equirectangular simulation grid. Longitude wraps; latitude is bounded. It maps
indices to `GeoPoint`, performs nearest and scalar sampling, reports cell
metrics, and computes slope.

[`SimulationSettings`](../../crates/worldtools_simulation/src/settings.rs)
controls atlas and climate resolution, plate and hotspot counts, geological
age, and iteration counts. Generation sanitizes these values before allocation
or iteration. Callers therefore get a valid snapshot even when settings come
from untrusted or experimental inputs.

### Stage graph

The exact composition is intentionally visible in
[`WorldSnapshot::generate`](../../crates/worldtools_simulation/src/snapshot.rs).
Its dependency flow is:

```text
base procedural terrain
        |
        v
tectonics (plates, crust, boundaries, uplift, deformation)
        |
        +------> provisional climate
        |                  |
        |                  v
        +----------> hydrology + erosion
                           |
                           v
                  recomputed climate
                           |
                           v
                  refreshed hydrology
                           |
                           v
                       glaciation
                           |
                           v
                  climate + hydrology refresh
                           |
              +------------+-------------+
              v                          v
       surface geology              final elevation
              |
              v
       soil + vegetation
              |
              v
          resources
```

The repeated climate and hydrology calls are deliberate coupling passes, not
duplicate initialization. Erosion/deformation changes elevation, and elevation
changes climate and water routing. Glaciation changes surface state again, so a
final refresh makes the stored channels agree with the evolved surface.

Stage implementations live under
[`crates/worldtools_simulation/src/stages`](../../crates/worldtools_simulation/src/stages/mod.rs):

| Stage module | Owns |
| --- | --- |
| `tectonics.rs` | Plate history, crust, boundaries, uplift, volcanic effects |
| `climate.rs` | Temperature, precipitation, seasonality, wind, aridity |
| `hydrology.rs` | Runoff, routing, rivers, lakes, erosion, sediment |
| `glaciation.rs` | Ice fraction/flux, glacial erosion, till, outwash, rebound |
| `geology.rs` | Lithology, rock age, sediment, ash, weathering |
| `ecology.rs` | Soil and vegetation/biome states |
| `resources.rs` | Dominant deposit, richness, depth, confidence, families |
| `multires.rs` | Resolution transfer helpers |
| `random.rs` and `math.rs` | Deterministic stage support math |

Mutable stage-specific structs are crate-private. The public output is one
`WorldSnapshot` whose arrays are shared immutable allocations.

### Snapshot sampling

The snapshot stores both baseline elevation and evolved elevation. Its detailed
point elevation is:

```text
detailed procedural baseline
  + (evolved atlas elevation - baseline atlas elevation)
```

This preserves high-frequency local terrain while applying coarse global
history. `sample_atlas_elevation` omits the local detail and is suitable for a
low-resolution fallback. `sample(point)` returns a full
[`WorldSample`](../../crates/worldtools_simulation/src/layers.rs), while
`sample_layer(point, layer)` returns only the four channels needed by one
renderer view.

Categorical fields use the nearest atlas cell; continuous fields use scalar
grid sampling. This distinction matters at plate, biome, soil, and lithology
boundaries.

The snapshot fingerprint includes the seed and both settings structures. Its
first eight bytes become `revision`, a convenient stable identifier. Render
cache correctness does not depend on revision uniqueness alone: world
replacement also advances a runtime epoch and replaces result channels.

## Render architecture

[`worldtools_render`](../../crates/worldtools_render/src/lib.rs) owns map
navigation, visible-page planning, CPU page generation, GPU resources, and
shader presentation. It consumes simulation/world contracts but never reads UI
types.

### View and projection

[`MapView`](../../crates/worldtools_render/src/view.rs) stores a normalized
equirectangular center and vertical span. Navigation systems pan and zoom it
from mouse input, constrained by [`MapViewport`](../../crates/worldtools_render/src/view.rs).
`MapNavigationSettings` lets the app decide whether primary-button drag means
pan; middle-button navigation remains a renderer concern.

[`plan_tiles`](../../crates/worldtools_render/src/projection.rs) combines the
view and physical viewport size into a `TilePlan`. The render quadtree uses
[`MapTileId`](../../crates/worldtools_render/src/projection.rs), an
equirectangular page id distinct from the cube-sphere `world::TileId`.
`MapTilePlacement` also carries an unwrapped x coordinate so pages remain
continuous across the date line.

### Streaming pipeline

The chained Update systems in
[`TileStreamingPlugin`](../../crates/worldtools_render/src/streaming.rs) execute:

```text
update_plan
    | calculate visible TilePlan
    v
receive_tiles
    | accept valid completed worker results
    v
invalidate_tiles
    | apply explicit MapTileInvalidation messages
    v
request_tiles
      launch missing visible pages up to MAX_IN_FLIGHT
```

`MapTileStreamer` owns:

- The active `Arc<WorldSnapshot>` and `WorldDataLayer`.
- A bounded resident Moka cache of `MapTileData`.
- In-flight ids, per-id revisions, and a bounded result channel.
- Counters and timings exposed as `TileStreamStats`.
- A `world_epoch` that advances whenever the complete snapshot is replaced.

Workers sample immutable snapshots on Bevy's async compute pool. A
[`MapTileData`](../../crates/worldtools_render/src/tile_data.rs) page contains
elevation samples plus four active-layer channels, including an apron for
filtering and normal calculation. The page can upload each payload as an image.

A completed result is accepted only when its revision equals the current tile
revision and its layer equals the streamer's active layer. World replacement is
stronger: it creates a new result channel, clears cache and bookkeeping, and
increments `world_epoch`. Old senders can finish, but no longer feed the active
receiver.

When an exact page is not resident, `best_available` walks toward the root and
returns the closest cached ancestor. This is the visual continuity path during
zooming or initial load.

### CPU pages to pixels

[`TileSurfacePlugin`](../../crates/worldtools_render/src/tile_surface.rs) reads
`VisibleMapTiles` and `MapTileStreamer` each frame:

```text
visible placement
  |
  +-- exact resident page? ---- yes --> use exact
  |
  +-- already rendered stale? - yes --> retain while replacement loads
  |
  +-- cached ancestor? -------- yes --> sample ancestor quadrant
  |
  +-- otherwise ----------------------> report missing
  |
  v
MapTileData
  |
  +-- elevation samples --> R32Float Image
  +-- layer channels ----> Rgba32Float Image
  |
  v
TerrainTileMaterial + quad entity --> worldtools_tile.wgsl --> pixels
```

The GPU cache keys by map tile id and checks `Arc::ptr_eq`; replacing CPU data
for the same id therefore creates new images. A world epoch change drains
rendered entities and GPU images before drawing the new world. Unused GPU pages
are retained only while the corresponding CPU page remains resident.

[`MapDisplaySettings`](../../crates/worldtools_render/src/display.rs) controls
shader mode, contour interval, opacity, relief/detail/boundary strength, and
lighting. Display changes update uniforms rather than regenerating the world.
The WGSL sources are
[`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl)
and the older full-surface
[`worldtools_terrain.wgsl`](../../crates/worldtools_render/src/worldtools_terrain.wgsl).

## UI architecture

[`worldtools_ui`](../../crates/worldtools_ui/src/lib.rs) is an editor shell plus
a communication model. It owns what the user has selected and what panels show;
it does not own or import a world snapshot.

`WorldToolsUiPlugin` installs egui, initializes the public model resources, adds
`RegenerateWorld` and `DebugCommand` messages, and draws the shell during
`EguiPrimaryContextPass`. The shell modules are under
[`src/shell`](../../crates/worldtools_ui/src/shell/mod.rs), while reusable visual
rules and widgets live in [`style.rs`](../../crates/worldtools_ui/src/style.rs)
and [`widgets.rs`](../../crates/worldtools_ui/src/widgets.rs).

Important resources include:

| Resource | Meaning |
| --- | --- |
| `EditorUiState` | Active tool, map view, layer, analysis drawer |
| `DocumentStatus` | Name and seed of the installed snapshot |
| `WorldGenerationDraft` | Editable seed not yet committed to the document |
| `GenerationStatus` | Idle/running/failed UI status |
| `MapViewport` | egui-measured logical and physical viewport rectangle |
| `MapPresentationSettings` | Per-layer visual style and lighting controls |
| `MapReadout` | Cursor, LOD, and metres-per-pixel status |
| `MapProbe` | Selected point and formatted layer readings |
| `LayerCapabilities` | Whether each UI layer is backed by native code |
| `DebugUiState` / `DebugTelemetry` | Debug controls and live measurements |
| `AnalysisStatus` | Current report name and editor-visible issues |

The full model is re-exported from
[`model.rs`](../../crates/worldtools_ui/src/model.rs). This makes the UI crate's
public boundary easy to inspect: app plugins exchange these resources and
messages without reaching into shell implementation.

### UI-to-render bridge

[`ViewportBridgePlugin`](../../apps/worldtools/src/viewport_bridge.rs) is the
intentional adapter between UI vocabulary and render vocabulary. Every Update
it copies:

- The egui viewport rectangle, input-blocked flag, and window scale into the
  renderer viewport.
- The active tool into primary-button navigation policy.
- `WorldLayer` through [`simulation_layer`](../../apps/worldtools/src/layers.rs)
  into the streamer's `WorldDataLayer`.
- Per-layer presentation controls into `MapDisplaySettings`.
- UI map-view/layer selections into the appropriate shader display mode.

Keeping this translation in the app prevents `worldtools_ui` and
`worldtools_render` from depending on each other.

## User workflows as data flow

### Regenerate a world

The UI emits `RegenerateWorld { seed }`. The app's
[`WorldGenerationPlugin`](../../apps/worldtools/src/generation.rs) owns the
workflow:

```text
UI edit WorldGenerationDraft
        |
        v
RegenerateWorld message
        |
        v
queue_regeneration
  copy current terrain/simulation settings from active snapshot
  set GenerationStatus::Running
  spawn WorldSnapshot::generate on AsyncComputeTaskPool
        |
        v
finish_regeneration polls Task on later frames
        |
        +-- not ready --> leave active world untouched
        |
        +-- ready ----> MapTileStreamer::replace_snapshot
                         update DocumentStatus and draft seed
                         clear MapProbe
                         set status Idle
```

Only one generation task is allowed at a time. Requests received while it runs
are logged and ignored. Because installation happens only after the task
completes, the visible document seed cannot disagree with the snapshot being
inspected.

### Select a display layer

The UI changes `EditorUiState.active_layer`. The viewport bridge maps it to
`WorldDataLayer` and calls `MapTileStreamer::set_active_layer`. That invalidates
resident and in-flight page identities, increments per-page revisions, and
causes future workers to sample the selected four-channel layer. Old pages can
remain visually stale until replacements arrive, but a mismatched layer is
shown in physical elevation mode rather than interpreted as the wrong overlay.

### Pan and zoom

The UI measures where the map is visible. The bridge publishes that rectangle
to render. Renderer navigation updates `MapView`; the streaming planner derives
a new visible plan from the view and physical resolution; missing pages are
requested; and tile-surface transforms place resident or ancestor pages in the
viewport.

### Inspect a point

[`WorldInteractionPlugin`](../../apps/worldtools/src/interaction.rs) converts
the pointer from window coordinates through the UI viewport and `MapView` to a
`GeoPoint`. On an Inspect-tool click,
[`capture_inspection`](../../apps/worldtools/src/interaction/probe.rs) samples
the streamer's snapshot directly. Elevation also estimates local slope with
four nearby samples. Other layers format the relevant `WorldSample` channels
into a UI-owned `LayerProbe`.

This path samples source data, not a GPU texture. Probe values therefore do not
depend on current page residency or fallback state.

## Analysis and diagnostics

### Reusable analysis

[`worldtools_analysis`](../../crates/worldtools_analysis/src/lib.rs) contains
pure computations over world and simulation data:

- Terrain distributions and physically scaled slope/ruggedness audits.
- Weighted aggregation across tiles.
- Same-face and cross-face seam comparison.
- Parent/child LOD consistency.
- Whole-snapshot numeric and process-coherence checks.

It does not choose files, commands, UI, or schedules. Those policies belong to
the lab or desktop app.

### In-app debug pipeline

[`WorldToolsDebugPlugin`](../../apps/worldtools/src/debug_tools/mod.rs) chains
event draining, telemetry synchronization, debug command handling, snapshot
capture, and asynchronous audits. `DebugCommand` is declared by the UI, but the
app decides what each command does to render resources or diagnostic work.

Process diagnostics are installed in
[`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs). They provide:

- Daily non-blocking file logs.
- A bounded tracing-event channel for the in-app debug window.
- Dropped-event accounting rather than blocking the runtime.
- Panic reports with build, process, environment, location, and backtrace.
- A stable default output directory at `.runtime/diagnostics`, overridable via
  `WORLDTOOLS_LOG_DIR`.

The default tracing filter can be replaced with `WORLDTOOLS_LOG`.

### Headless lab

[`worldtools_lab`](../../tools/worldtools_lab/src/main.rs) exercises the same
engine-independent crates without Bevy:

| Command | Purpose |
| --- | --- |
| `overview` | Render a PNG from continuous terrain samples |
| `tile` | Render and audit one cube-sphere tile |
| `verify` | Check seams, LOD, and statistics at one level |
| `sweep` | Compare exhaustive six-face aggregates across seeds |
| `world` | Generate and audit one coupled simulation snapshot |

The CLI contract is in [`args.rs`](../../tools/worldtools_lab/src/args.rs).
Report serialization lives in [`report.rs`](../../tools/worldtools_lab/src/report.rs).

### Development harness

[`xtask`](../../xtask/src/main.rs) is deliberately independent of runtime
crates. It owns host capability reports, deterministic TOML debug cases,
reproduction/capture artifacts, sequential verification profiles, and debugger
script generation. Debug case schema is in
[`case.rs`](../../xtask/src/case.rs); captured-run orchestration is in
[`reproduce.rs`](../../xtask/src/reproduce.rs).

Keeping this harness separate means it can diagnose a broken application build
without linking application code into the harness itself.

## Ownership boundaries

When changing the project, use this table to decide where code belongs.

| Question | Correct owner |
| --- | --- |
| How is a seed deterministically split? | `worldtools_world::seed` |
| How does a cube-sphere sample map to a direction? | `worldtools_world::tile/geo` |
| What channels does climate or hydrology produce? | `worldtools_simulation` |
| In what order are coupled stages run? | `WorldSnapshot::generate` |
| Which map pages should be visible? | `worldtools_render::projection` |
| How are pages cached, invalidated, or uploaded? | `worldtools_render` |
| What does an editor control or panel mean? | `worldtools_ui` |
| How does a UI selection affect the renderer? | `apps/worldtools` bridge |
| How is user-triggered generation scheduled? | `apps/worldtools::generation` |
| How are numerical properties audited? | `worldtools_analysis` |
| How is a headless run exposed as a CLI/report? | `worldtools_lab` |
| How are deterministic reproductions captured? | `xtask` |

Avoid putting render-specific page ids into `worldtools_world`, Bevy resources
into simulation, or snapshot layout knowledge into UI. A cross-crate workflow
usually needs a small neutral contract in the owning crate plus an app adapter.

## Concurrency and consistency

There are two main background workloads:

1. Complete snapshot generation, represented by one Bevy `Task<GeneratedWorld>`
   in `GenerationCoordinator`.
2. Per-page render generation, launched through `AsyncComputeTaskPool` and
   returned over a bounded crossbeam channel.

Both workloads receive owned settings and shared immutable snapshots. They do
not mutate the active snapshot from worker threads. ECS systems poll or drain
results and perform state installation on the main world.

The consistency mechanisms cover different scopes:

| Mechanism | Invalidates |
| --- | --- |
| Settings/seed fingerprint | Identity of generated world content |
| Per-tile revision | Work superseded by page invalidation |
| Active layer check | Work generated for another overlay |
| `world_epoch` | GPU/entity state from another snapshot |
| Replaced result channel | Late worker sends from another snapshot |
| `Arc` immutable storage | Partial or racy reads of generated channels |

## Adding a feature safely

### Add a simulation layer or channel

1. Add process state and deterministic stage logic under simulation `stages`.
2. Store the finished channel in `WorldSnapshot` and expose semantic sampling.
3. Extend `WorldDataLayer` and `WorldSample` where appropriate.
4. Extend `sample_layer` with a stable four-channel shader contract.
5. Add the corresponding UI `WorldLayer` and preserve the tested ordering in
   [`layers.rs`](../../apps/worldtools/src/layers.rs).
6. Add render display mode/shader interpretation and probe formatting.
7. Add analysis checks for numerical validity and cross-layer coherence.

### Add an editor action

Put visual state and a message/command in `worldtools_ui`. Implement effects in
an app plugin or the crate that owns the affected mechanism. Do not make an
egui panel reach into renderer internals.

### Add a rendering-only style

Add UI presentation state if it is user-editable, translate it in the viewport
bridge, and implement the uniform/shader behavior in render. Presentation-only
changes should not invalidate `WorldSnapshot` or regenerate CPU pages unless
the page's sampled data truly changes.

### Add a diagnostic

Put pure measurements in `worldtools_analysis`, orchestration in an app debug
module or `worldtools_lab`, and output policy at that outer layer. Keep captures
under `.runtime` or `.debug` rather than source directories.

## Fast mental model

When reading unfamiliar code, ask three questions:

1. **Is this authoritative or derived?** `WorldSnapshot` and editor intent are
   authoritative; streamed pages, GPU images, telemetry, and formatted probes
   are derived.
2. **Is this a domain mechanism or integration policy?** Domain mechanisms
   live in crates; translation and scheduling between crates live in the app.
3. **What makes stale work harmless?** Look for snapshot immutability, revision,
   layer, epoch, or channel replacement at every asynchronous boundary.

Those answers explain most of the runtime without needing to memorize every
Bevy system.
