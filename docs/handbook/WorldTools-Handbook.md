# WorldTools Complete Offline Handbook

This single-file edition is generated from the maintained chapters beside it.

---

# WorldTools Offline Handbook

This handbook is a self-contained guide to the WorldTools codebase. Every link stays inside this repository. No page requires an internet connection, a documentation server, JavaScript, or a downloaded asset.

Start with [Getting Started](getting-started.md), then follow one of the reading routes below.

For a reader that handles one long file better than a linked folder, open [WorldTools-Handbook.md](WorldTools-Handbook.md). It contains every chapter in reading order.

## Reading routes

### One-hour orientation

1. [Getting Started](getting-started.md)
2. [Architecture](architecture.md)
3. [Crate Map](crate-map.md)
4. [Glossary](glossary.md)

### Understand world generation

1. [Simulation](simulation.md)
2. [Data Layers](data-layers.md)
3. [Code Reading Tours](reading-tours.md#tour-2-follow-one-seed-into-a-world)

### Understand the application

1. [Rendering and UI](rendering-ui.md)
2. [Code Reading Tours](reading-tours.md#tour-3-follow-one-visible-map-tile)
3. [Debugging](debugging.md)

### Prepare to contribute

1. [Architecture](architecture.md)
2. [Code Reading Tours](reading-tours.md)
3. [Debugging](debugging.md)
4. Read the repository [agent and engineering constraints](../../AGENTS.md)

## Handbook contents

| Chapter | Purpose |
| --- | --- |
| [Getting Started](getting-started.md) | Build, run, navigate, and locate the important entry points |
| [Architecture](architecture.md) | Runtime composition, ownership boundaries, and end-to-end data flow |
| [Crate Map](crate-map.md) | Responsibility and public surface of every workspace member |
| [Simulation](simulation.md) | Deterministic generation stages and physical coupling |
| [Data Layers](data-layers.md) | Meaning and representation of every generated map layer |
| [Rendering and UI](rendering-ui.md) | Infinite map, LOD streaming, WebGPU shader, egui shell, presentation controls |
| [Debugging](debugging.md) | Diagnostics, deterministic cases, captures, tests, and live inspection |
| [Code Reading Tours](reading-tours.md) | Guided paths through real source for common questions |
| [Glossary](glossary.md) | Project-specific vocabulary and important type names |
| [Design Invariants](design-invariants.md) | Contracts that code and data-model changes must preserve |
| [Study Plan](study-plan.md) | Timed reading plan and self-check exercises |

## Repository at a glance

```text
project-bombo/
|-- apps/worldtools/              native application and integration systems
|-- crates/worldtools_world/      deterministic coordinates, seeds, cube tiles, base terrain
|-- crates/worldtools_simulation/ coupled planetary history and data layers
|-- crates/worldtools_render/     wrapped 2D map, LOD pages, WebGPU presentation
|-- crates/worldtools_ui/         egui editor state and shell
|-- crates/worldtools_analysis/   quantitative generation-quality audits
|-- tools/worldtools_lab/         offline terrain experiments and reports
|-- xtask/                        reproducible diagnostics and validation harness
`-- docs/handbook/                this offline handbook
```

## Core mental model

WorldTools is not a scene made from terrain meshes. It is a deterministic world-data system presented through a streamed two-dimensional map.

```text
explicit seed + generation settings
                |
                v
       WorldSnapshot (global atlas)
                |
      geographic point sampling
                |
                v
       MapTileData (LOD page)
                |
       CPU -> GPU image upload
                |
                v
   WebGPU material + WGSL presentation
                |
                v
       egui viewport and controls
```

The important separation is:

- **World and simulation code decides what exists.**
- **Renderer code decides what data is resident and how it is drawn.**
- **UI code owns user intent but not simulation or renderer internals.**
- **The application crate translates between those contracts.**

## Offline guarantee

The handbook itself is plain UTF-8 Markdown. Relative links target checked-in source files. Code snippets are explanatory copies; the linked Rust source remains authoritative. Commands may require dependencies already present in the Cargo cache, but reading the handbook requires only a text editor or Markdown viewer.

Validate the offline contract and every local link with:

```powershell
powershell -ExecutionPolicy Bypass -File docs/handbook/check-offline.ps1
```

Rebuild the single-file edition after editing a chapter with:

```powershell
powershell -ExecutionPolicy Bypass -File docs/handbook/build-single-file.ps1
```

---

# Getting Started

## What WorldTools is

WorldTools is a native Rust and Bevy application for generating and inspecting deterministic, physically coupled worlds. Its current product loop is:

1. Choose a seed.
2. Generate a complete world history snapshot.
3. Explore the map at multiple levels of detail.
4. Switch between elevation, tectonics, hydrology, climate, soil, vegetation, geology, and resources.
5. Inspect values at a geographic point.
6. Use diagnostics and analysis to distinguish presentation defects from generation defects.

The application deliberately excludes intelligent life, settlements, politics, and a fantasy-specific ruleset. Its domain is the natural world.

## Prerequisites

- Rust `1.95.0`, pinned by [`rust-toolchain.toml`](../../rust-toolchain.toml)
- A working Windows MSVC toolchain on Windows, including the Visual C++ linker
- A WebGPU-capable graphics adapter and driver
- PowerShell for the commands in this handbook; the Cargo commands are equivalent on Linux

WorldTools has no renderer fallback. Failure to initialize WebGPU is a real startup failure, not a reason to silently use a different rendering path.

## First run

From the repository root:

```powershell
cargo run -p worldtools
```

The executable is defined in [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs). The application wires together the UI, simulation, rendering, interaction, diagnostics, and optional live-debug plugins there.

## First five minutes in the application

- Drag with the primary navigation input to pan.
- Use the wheel or equivalent input to zoom around the cursor.
- Keep panning horizontally: longitude is intentionally unwrapped and the world repeats.
- Select a data layer in **World Explorer**.
- Use **Presentation** in the inspector to change opacity, relief, shadow strength, detail, borders, lighting, and legend visibility.
- Select **Inspect**, then click the map to pin a geographic sample.
- Press `F12` to open the diagnostics window.
- Change the seed and choose **Regenerate World** to rebuild every coupled layer.

The UI contract is declared in [`crates/worldtools_ui/src/model`](../../crates/worldtools_ui/src/model). Application systems convert that intent into renderer and simulation operations.

## Validate the checkout

```powershell
cargo xtask doctor
cargo xtask check quick
```

`doctor` reports the execution environment and debugging capabilities. `check quick` runs formatting, type checking, strict Clippy, and the workspace library tests. See [Debugging](debugging.md) before diagnosing a runtime defect.

## Recommended editor setup

Use an editor with rust-analyzer and WGSL syntax support. These are conveniences, not handbook dependencies. The fastest navigation technique without an IDE is ripgrep:

```powershell
rg "struct WorldSnapshot" crates
rg "impl Plugin" apps crates
rg "WorldDataLayer" apps crates
rg "@fragment" crates/worldtools_render
```

## Build profiles

The workspace profiles live in [`Cargo.toml`](../../Cargo.toml):

- `dev` keeps application debug information and optimizes dependencies.
- `test` keeps limited debug information.
- `release` uses thin LTO, one codegen unit, and strips symbols.

Use debug builds for ordinary iteration. Use optimized builds for performance conclusions:

```powershell
cargo run -p worldtools --release
```

## Important entry points

| Question | Start here |
| --- | --- |
| Which plugins make the app? | [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs) |
| How is a new seed generated? | [`apps/worldtools/src/generation.rs`](../../apps/worldtools/src/generation.rs) |
| What does a world snapshot contain? | [`crates/worldtools_simulation/src/snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs) |
| In what order do simulation stages run? | [`crates/worldtools_simulation/src/stages/mod.rs`](../../crates/worldtools_simulation/src/stages/mod.rs) |
| How are visible map tiles selected? | [`crates/worldtools_render/src/projection.rs`](../../crates/worldtools_render/src/projection.rs) |
| How are pages generated and cached? | [`crates/worldtools_render/src/streaming.rs`](../../crates/worldtools_render/src/streaming.rs) |
| How does the terrain look 2.5D? | [`crates/worldtools_render/src/worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl) |
| Where is editor state defined? | [`crates/worldtools_ui/src/model/editor.rs`](../../crates/worldtools_ui/src/model/editor.rs) |
| How does UI state reach the renderer? | [`apps/worldtools/src/viewport_bridge.rs`](../../apps/worldtools/src/viewport_bridge.rs) |
| How do I inspect runtime evidence? | [`apps/worldtools/src/debug_tools`](../../apps/worldtools/src/debug_tools) and [`xtask/src`](../../xtask/src) |

## A useful debugging distinction

Before editing code, classify the defect:

| Symptom | Likely owner |
| --- | --- |
| The sampled numeric data is wrong everywhere | simulation stage or snapshot construction |
| Point inspection is right but the layer looks wrong | renderer sampling, palette, or shader |
| Detail changes at tile borders | tile generation, halo, fallback LOD, or shader footprint |
| A layer selection does nothing | UI-to-render bridge or streaming invalidation |
| The same seed changes between runs | seed derivation, iteration order, or non-deterministic parallel reduction |
| Only close zoom looks blocky | source atlas resolution or tile-local detail generation |

This distinction prevents visual tuning from concealing incorrect world data.

---

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

---

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

---

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

---

# Design Invariants

These are the rules that make WorldTools coherent. They are stricter than coding style because violating one can produce plausible-looking but incorrect worlds.

## Determinism is an API contract

Every generated result must be a function of explicit settings, geographic coordinates, and domain-separated seed data.

Consequences:

- Do not read wall-clock time in generation code.
- Do not use a process-global random stream.
- Do not let thread scheduling choose reduction order when floating-point order affects output.
- Derive random keys by subsystem and feature identity.
- Preserve fixed iteration counts and stable traversal order where convergence is not exact.
- Add tests that compare bits when bit determinism is required.

Primary sources: [`seed.rs`](../../crates/worldtools_world/src/seed.rs), [`random.rs`](../../crates/worldtools_simulation/src/random.rs), and snapshot determinism tests in [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs).

## Typed domain state comes before GPU channels

Simulation stages should communicate using named domain fields. Four-channel RGBA pages are a renderer transport contract, not the simulation model.

Consequences:

- Do not make a stage read â€œchannel Z.â€
- Define units and ranges in typed structures.
- Pack channels at the snapshot/render boundary.
- Keep probe formatting based on typed samples, not colors.
- Test typed and direct channel sampling against each other.

Primary source: [`layers.rs`](../../crates/worldtools_simulation/src/layers.rs).

## Categorical and continuous data are different

Plate IDs, soil orders, biome classes, lithology, and deposit families are identities. Elevation, temperature, precipitation, discharge, richness, and confidence are measurements.

Consequences:

- Never bilinearly interpolate IDs.
- Use explicit neighbor comparison or a boundary-distance field for category borders.
- Use bounded interpolation for physical measurements when overshoot would be invalid.
- Keep palette lookup separate from numeric normalization.

Primary sources: [`grid.rs`](../../crates/worldtools_simulation/src/grid.rs) and [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl).

## Stage order is model semantics

The simulation pipeline is not a bag of independent filters. Each stage consumes a world already changed by earlier processes.

Consequences:

- Reordering stages requires scientific and product justification.
- A refresh pass is deliberate when a mutation invalidates derived fields.
- Avoid hidden coupling through globals; pass or own required fields explicitly.
- Document whether a stage mutates surface height, hydrology, or only its own output.

Primary source: [`stages/mod.rs`](../../crates/worldtools_simulation/src/stages/mod.rs).

## Global processes cannot be faked per tile

Drainage basins, major rivers, climate circulation, ice sheets, and broad tectonic structure cross page boundaries.

Consequences:

- Solve global or regional context before tile-local presentation.
- Use halos for local filters, not as a substitute for global state.
- Do not derive independent river networks inside visible pages.
- Test adjacency and parent-child continuity independently.

## World identity protects asynchronous work

Tile generation and complete-world generation can finish after the user has requested different data.

Consequences:

- Tag results with world epoch/revision and active layer.
- Reject stale results at acceptance, not only when scheduling.
- Do not reuse a GPU page merely because its coordinates match.
- Preserve the camera across regeneration while invalidating old data.

Primary source: [`streaming.rs`](../../crates/worldtools_render/src/streaming.rs).

## Canonical data, unwrapped presentation

The world repeats horizontally in the flat map, but generated data should not be duplicated for every visible copy.

Consequences:

- Wrap canonical tile X for caching and generation.
- Retain unwrapped placement X for screen position.
- Anchor shader patterns to canonical geographic coordinates.
- Treat the antimeridian as ordinary adjacency.

Primary sources: [`view.rs`](../../crates/worldtools_render/src/view.rs) and [`projection.rs`](../../crates/worldtools_render/src/projection.rs).

## Presentation must not alter world data

Opacity, palettes, hillshade, borders, labels, contour interval, and sun direction are visual state.

Consequences:

- Apply them through uniforms or UI overlay state.
- Do not regenerate a world for a presentation change.
- Keep generated values stable while comparing views.
- Make display sanitization explicit.

Primary sources: [`display.rs`](../../crates/worldtools_render/src/display.rs) and [`presentation.rs`](../../crates/worldtools_ui/src/model/presentation.rs).

## Renderer footprints require matching halos

Any shader or CPU filter that reads neighboring samples has a maximum radius.

Consequences:

- Keep `MAP_TILE_APRON` at least as large as the largest page-space sample offset.
- Test exact shared boundaries.
- Inspect fallback LOD transitions separately from same-LOD seams.
- Avoid clamp-to-edge as an accidental tile-boundary policy.

Primary sources: [`projection.rs`](../../crates/worldtools_render/src/projection.rs), [`tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs), and [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl).

## Evidence precedes fixes

Runtime bugs should be reproduced and captured before editing.

Consequences:

- Use a fixed seed and case definition.
- Retain logs, metadata, traces, and snapshots.
- Rank no more than three hypotheses.
- Run the cheapest experiment that separates them.
- Repeat the original reproduction after the fix.

Primary sources: [Debugging](debugging.md), [`xtask/src`](../../xtask/src), and [`apps/worldtools/src/debug_tools`](../../apps/worldtools/src/debug_tools).

---

# World Generation and Simulation

This chapter is a source-level guide to how WorldTools turns a seed and two
settings records into an immutable, globally sampleable world. It describes the
implemented model, not an idealized Earth simulator. Read it alongside
[`data-layers.md`](data-layers.md) for the exact public output contract.

## The two generation systems

WorldTools combines two deliberately different spatial systems:

1. `worldtools_world` supplies continuous procedural terrain. A
   [`TerrainGenerator`](../../crates/worldtools_world/src/terrain.rs) can sample
   any direction on the sphere and can materialize cube-sphere terrain tiles.
2. `worldtools_simulation` runs coupled, long-timescale processes on a compact
   global latitude/longitude atlas. A
   [`WorldSnapshot`](../../crates/worldtools_simulation/src/snapshot.rs) owns the
   resulting arrays and samples them at arbitrary geographic points.

The final detailed elevation is a composition rather than a single stored
height field:

```text
detailed elevation(point)
    = procedural terrain(point)
    + atlas deformation(point)

atlas deformation(point)
    = evolved atlas elevation(point)
    - baseline atlas elevation(point)
```

This is the key to the apparent resolution mismatch. Tectonics, erosion,
deposition, and glaciation operate at atlas resolution, while procedural local
relief survives below an atlas cell. See `WorldSnapshot::sample_elevation` in
[`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs).

## Composition root and exact stage order

The authoritative pipeline is `WorldSnapshot::generate`. Do not infer the order
from module names: several stages deliberately run more than once.

```text
WorldSeed + TerrainSettings + SimulationSettings
                         |
                         v
              sanitize SimulationSettings
                         |
                         v
                  build AtlasGrid
                         |
                         v
                  [1] tectonics
                         |
                         v
            [2] provisional climate
                         |
                         v
       [3] hydrology + fluvial erosion
               (mutates elevation)
                         |
                         v
                 [4] climate again
                         |
                         v
            [5] hydrology refresh
                         |
                         v
      [6] glacial advance and retreat
       (mutates elevation + hydrology)
                         |
                         v
                 [7] climate again
                         |
                         v
            [8] hydrology refresh
                         |
                         v
              [9] surface geology
                         |
                         v
            [10] soil + vegetation
                         |
                         v
                 [11] resources
                         |
                         v
             final slope + fingerprint
                         |
                         v
                immutable WorldSnapshot
```

The repeated climate/hydrology passes are a bounded coupling strategy:

- Provisional climate provides precipitation and temperature for erosive
  hydrology.
- Erosion and deposition alter the surface, so climate is recalculated.
- The hydrology refresh recalculates runoff, rivers, lakes, and wetness from the
  new climate without repeating fluvial erosion.
- Glaciation then alters relief and adds meltwater and sediment.
- A final climate and hydrology refresh makes those public fields agree with the
  glacially modified surface.

This is not a convergence loop. It is a fixed sequence with deterministic cost
and deterministic data dependencies.

## Determinism and seed flow

The root [`WorldSeed`](../../crates/worldtools_world/src/seed.rs) is a portable
`u64`. It derives domain-separated 256-bit `SeedKey` values with BLAKE3 derive-key
mode, a fixed context string, length-prefixed domain bytes, and little-endian
seed bytes. Tile keys additionally hash the stable bytes of a `TileId`. This
avoids Rust's randomized `Hash` state and prevents one subsystem's random draws
from shifting another subsystem.

```text
WorldSeed(42)
    |
    +-- key("terrain.continental") ------> FastNoise seed
    +-- key("terrain.mountain") ---------> independent FastNoise seed
    +-- key("terrain.detail") -----------> independent FastNoise seed
    +-- key("simulation.plates.v1") -----> StableRng stream
    +-- key("simulation.hotspots.v1") ---> StableRng stream
    +-- key("simulation.climate.wave.v1") -> circulation phase
    +-- key("simulation.resources.v2") --> resource district fields
```

[`StableRng`](../../crates/worldtools_simulation/src/random.rs) is a small,
explicit SplitMix64-style generator. `hash_unit(seed, index, domain)` converts a
stable index and domain into a reproducible scalar. Tectonic plate and hotspot
creation consume private stable streams. Resource districts use indexed hashes,
so evaluation order does not affect them. Climate uses a derived seed only for
the phase of its planetary wave.

Rayon parallelism is used only where each output cell is independently mapped
into its own result slot. Reduction-sensitive stages use explicit sequential
passes or deterministic ordering. Hydrology sorts with `f32::total_cmp` and an
index tie-break; priority flooding also includes the index in its ordering.

The snapshot fingerprint hashes:

- the root seed;
- every `TerrainSettings` floating-point bit pattern;
- every sanitized `SimulationSettings` integer.

The first eight fingerprint bytes form the snapshot `revision`. The revision is
a configuration identity, not a hash of every generated output array. Changing
an implementation while retaining settings can therefore change world content
without changing this revision. Versioned seed domains and the snapshot context
must be bumped intentionally when compatibility requires it.

## Base terrain

[`TerrainGenerator`](../../crates/worldtools_world/src/terrain.rs) uses three
3D sphere-space OpenSimplex2S fields:

- `continental`: fBm, five octaves; determines ocean versus land and broad
  lowland height;
- `mountain`: ridged fractal, five octaves; gated to inland ridge regions;
- `detail`: fBm, four octaves; adds small local relief on land.

Sampling normalized 3D directions instead of planar coordinates makes the
field continuous around longitude and across cube-face edges. Ocean height is a
shaped depth below sea level. Land combines lowland relief, a gated mountain
term, and local detail.

The terrain crate can also generate 256 by 256-cell cube-sphere tiles with one
sample apron. The stored tile is 259 by 259 samples: 257 vertex samples plus one
apron sample on every side. Shared samples are computed from global geometric
coordinates, so adjacent tiles and parent/child sample points match exactly.
Read [`tile.rs`](../../crates/worldtools_world/src/tile.rs),
[`geo.rs`](../../crates/worldtools_world/src/geo.rs), and the terrain tests in
[`terrain.rs`](../../crates/worldtools_world/src/terrain.rs) together.

## Global atlas

[`AtlasGrid`](../../crates/worldtools_simulation/src/grid.rs) is a cell-centered
equirectangular grid. Its flat storage index is `y * width + x`. Cell centers
cover longitude `[-pi, pi)` and latitude from north to south without placing
samples exactly on a pole.

Neighborhood behavior is asymmetric by design:

- X coordinates wrap, making longitude periodic.
- Y coordinates clamp at the first and last row.
- Scalar fields use bilinear interpolation.
- Categorical fields use the nearest cell.
- East-west cell size shrinks with `cos(latitude)` but is floored at `0.02` in
  metric calculations to avoid a singularity near the poles.

The default atlas is 512 by 256 cells. Climate defaults to a coarser 128 by 64
grid. `SimulationSettings::sanitized` clamps all dimensions, stage counts, and
iteration counts before use. The exact defaults and allowed ranges live in
[`settings.rs`](../../crates/worldtools_simulation/src/settings.rs).

### Coarse climate and fine terrain

When climate resolution differs from atlas resolution,
[`multires.rs`](../../crates/worldtools_simulation/src/stages/multires.rs):

1. area-weights fine elevation and land fraction into coarse cells;
2. preserves the maximum elevation as a barrier signal;
3. runs climate on a resolved coarse surface;
4. bilinearly upsamples scalar climate fields;
5. reapplies a local 6.3 C/km lapse-rate correction against fine relief;
6. applies bounded windward precipitation enhancement and leeward rain shadow
   using physical slope and wind direction.

This retains narrow mountain influence without paying full climate-transport
cost at atlas resolution.

## Stage 1: tectonics

[`tectonics.rs`](../../crates/worldtools_simulation/src/stages/tectonics.rs)
creates randomly distributed plate centers and angular-motion axes. A plate is
also assigned continental/oceanic character and a crustal age. Each atlas cell
is owned by the plate center with the greatest spherical dot product; the
second-nearest plate defines the active boundary.

Relative plate velocities are decomposed into convergence, divergence, and
transform shear. Continental character plus the strongest motion component
classifies the boundary as intraplate, collision, subduction arc, island arc,
ocean ridge, continental rift, or transform.

The stage also reconstructs shifted paleo plate centers from geological age.
Present/paleo ownership becomes a terrane signature, and proximity to old
boundaries produces inherited sutures. Hotspot tracks contribute volcanism
independently of current plate boundaries.

Base procedural elevation is modified by crustal buoyancy, collision uplift,
spreading relief, volcanic construction, inherited relief, and transform
relief. The output initializes crust thickness, metamorphic grade, lithology,
rock age, sediment cover, and volcanic ash. Both the untouched atlas baseline
and mutable evolved elevation are retained.

Important simplification: plates are spherical Voronoi regions with analytic
motion signals. They do not deform polygon meshes, conserve crustal area, solve
mantle convection, or integrate plate positions through time.

## Stages 2, 4, and 7: climate

[`climate.rs`](../../crates/worldtools_simulation/src/stages/climate.rs) computes
annual-mean climate, not daily weather. Temperature combines latitude-based
insolation, elevation lapse rate, continentality, and polar ocean moderation.
Seasonality increases with latitude and distance inland.

Prevailing winds approximate three circulation bands:

- tropical trade winds;
- mid-latitude westerlies;
- polar easterlies.

A seed-dependent planetary wave and meridional circulation break perfect zonal
symmetry. Winds are annual prevailing values in metres per second.

Moisture starts high over ocean and low over land. A fixed number of transport
iterations gathers moisture from upwind zonal and meridional cells, adds ocean
evaporation, and precipitates moisture according to relief, cold, and local
moisture. The final fields are temperature, precipitation, seasonality, wind,
aridity, and a coarse Koppen-like class.

The classifier is intentionally compact. It uses annual mean temperature,
precipitation-derived aridity, and seasonality thresholds; it is not the full
Koppen-Geiger rule set.

## Stages 3, 5, and 8: hydrology

[`hydrology.rs`](../../crates/worldtools_simulation/src/stages/hydrology.rs)
derives land runoff from precipitation, aridity, and thaw. Its first invocation
also performs iterative fluvial surface evolution.

For each erosion iteration:

1. Priority flooding raises closed depressions just enough to produce a
   drainable surface. A 0.05 m deterministic gradient resolves flats.
2. Each land cell chooses the steepest of eight neighbors on the filled
   surface, with physical east-west/north-south distances.
3. Runoff is accumulated from high to low filled elevation.
4. Stream power and local slope incise channels; runoff also causes sheet wash.
5. A slope-dependent fraction deposits downstream. Strong rivers entering
   shallow ocean spread a small depositional fan across neighbors.
6. Elevation deltas are applied as a coherent iteration boundary.

After erosion, routing is rebuilt. River strength is a normalized log-flow
signal. Lakes combine depression fill depth and water supply. Wetness combines
runoff, river strength, lakes, and poorly drained low slope.

The later `refresh_after_climate` calls reroute and recompute runoff, rivers,
lakes, and wetness but preserve accumulated erosion and sediment. This prevents
the coupling passes from applying erosion multiple times.

## Stage 6: glaciation

[`glaciation.rs`](../../crates/worldtools_simulation/src/stages/glaciation.rs)
models one long-timescale advance and retreat. Annual temperature plus
seasonality yields winter and summer temperatures. A 9 C colder glacial summer,
winter snow, summer survival, and precipitation supply define maximum local ice.

Ice is routed downhill and accumulated into a normalized flux. Erosion depends
on ice, flux, slope, lithology hardness, geological history, and the configured
glacial integration count. At ice margins the stage deposits till and outwash,
adds meltwater to hydrology, and calculates bounded isostatic rebound.

Glacial erosion, rebound, and deposition mutate elevation. Sediment, runoff,
wetness, and river strength are also updated. The subsequent climate and
hydrology passes are therefore part of the glacial coupling contract.

Important simplification: `glacial_iterations` scales erosion magnitude by a
square root; the code does not execute that many dynamic ice-sheet time steps.
There is no explicit ice thickness, basal temperature, sea-level change, or
mass-conserving shallow-ice solver.

## Stage 9: surface geology

[`geology.rs`](../../crates/worldtools_simulation/src/stages/geology.rs) refines
the tectonic lithology into a surface classification. Volcanic sources emit ash
into a 14-step wind advection and precipitation washout pass. Weathering grows
with warmth and moisture and falls on steep slopes. Hydrologic deposition and a
fraction of erosion produce the exposed sediment measure.

Rule precedence then selects volcanic arc, unconsolidated sediment, warm
shallow-water carbonate, sedimentary, metamorphic, exposed plutonic, or the
original tectonic lithology. Rock age is inherited from tectonics.

This is surface geology, not a volumetric stratigraphic column. Sediment and ash
are scalar thickness-like summaries and lithology is one dominant category.

## Stage 10: soil and vegetation

Soil and vegetation are computed together in
[`ecology.rs`](../../crates/worldtools_simulation/src/stages/ecology.rs) because
their inputs and provisional biomass calculation are shared.

Soil combines slope retention, weathering, sediment, ash, fluvial context,
waterlogging, erosion loss, climate, and provisional organic productivity.
This produces depth, fertility, clay, organic fraction, drainage, and one soil
class. Classification order matters: ocean and bare rock are checked before
climate- and material-specific types.

Biome classification combines ocean/ice state, temperature, precipitation,
aridity, wetness, elevation, slope, seasonality, rivers, and lakes. Each biome
has characteristic canopy and grass cover, then growth support scales it by
fertility, moisture, and temperature. Biomass and fire frequency are derived
from those covers.

This is a one-way equilibrium ecology pass. Vegetation does not feed back into
climate, erosion, soil water, or fire succession, and there are no species or
time-varying populations.

## Stage 11: resources

[`resources.rs`](../../crates/worldtools_simulation/src/stages/resources.rs)
supports 15 deposit types divided into metallic, energy, and industrial groups.
Every deposit model defines host and occurrence thresholds, richness scaling,
district spacing and footprint, frequency, and structural family.

Generation has three conceptual layers:

1. Process scores evaluate whether each cell is a plausible geological,
   climatic, hydrologic, soil, and ecological host.
2. Seeded districts and coherent structural fabric localize those broad host
   regions into finite occurrences. District dimensions scale against a
   reference 192-row atlas so changing resolution does not wildly change their
   represented geographic size.
3. The richest qualifying occurrence becomes the dominant deposit. Depth and
   evidence confidence are derived separately. Group prospectivity retains the
   best metallic, energy, and industrial signal even where no deposit crosses
   the mapped-occurrence threshold.

Resources are procedural prospectivity, not reserve estimates. Richness and
confidence are normalized signals; they do not represent tonnage, grade,
economic viability, or survey uncertainty in calibrated units.

## Snapshots, sampling, and rendering

Once all stages finish, mutable vectors are moved into `Arc<[T]>` fields inside
`WorldSnapshot`. The public object exposes no mutation. Sharing an `Arc` gives
tile workers, probes, and analysis code one consistent world revision.

`WorldSnapshot::sample(point)` returns every typed field. Continuous scalars are
bilinearly sampled; categorical IDs use the nearest atlas cell. `sample_layer`
returns only the four renderer channels for one selected layer. Consult
[`data-layers.md`](data-layers.md) before treating those packed channels as a
complete scientific record.

The renderer's [`MapTileData`](../../crates/worldtools_render/src/tile_data.rs)
resamples a snapshot into equirectangular map pages. Each page stores an R32
elevation image and an RGBA32F selected-layer image. Level-zero fallback pages
use atlas elevation without procedural detail for quick initial coverage; finer
pages use composed detailed elevation. The
[`MapTileStreamer`](../../crates/worldtools_render/src/streaming.rs) binds async
results to layer and revision so stale work cannot enter the active cache.

## Model boundaries and common misreadings

- Deterministic means identical explicit inputs and implementation produce the
  same bits. It does not promise identical results after algorithm or dependency
  changes unless compatibility is actively maintained.
- The atlas is equirectangular. Cell count is not cell area; polar cells cover
  less area. Some algorithms compensate with cosine weights, but not every
  empirical classification does.
- Longitude wraps; latitude clamps. Polar topology is a collapsed boundary in
  reality but a row of cells here.
- Coupling is staged, not solved to equilibrium. There is no iterative
  convergence criterion across climate, water, ice, geology, and ecology.
- Terrain detail is procedural decoration plus atlas deformation. Fine relief
  does not participate in atlas hydrology or climate.
- Most normalized values are heuristic indices. Read field names and probe
  formatting before assigning units.
- Classification fields are nearest-neighbor discontinuous by design.
- Snapshot generation materializes many full-atlas vectors. Resolution affects
  both CPU time and memory approximately with `width * height`; iteration counts
  multiply the cost of their corresponding stages.
- The public snapshot is immutable, but generation itself mutates the tectonic
  elevation and sediment vectors as coupled stages run.
- There is no snapshot serialization format in this crate. A snapshot is
  regenerated from seed and settings in memory.

## Practical code-reading routes

### Fast architecture route

1. [`lib.rs`](../../crates/worldtools_simulation/src/lib.rs): public boundary.
2. [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs): ownership,
   exact pipeline, sampling, and tests.
3. [`settings.rs`](../../crates/worldtools_simulation/src/settings.rs): cost and
   fidelity controls.
4. [`grid.rs`](../../crates/worldtools_simulation/src/grid.rs): indexing,
   topology, interpolation, and metric derivatives.
5. [`data-layers.md`](data-layers.md): externally observable data contract.

### Follow elevation through the system

1. [`terrain.rs`](../../crates/worldtools_world/src/terrain.rs): continuous base.
2. [`tectonics.rs`](../../crates/worldtools_simulation/src/stages/tectonics.rs):
   baseline capture and uplift.
3. [`hydrology.rs`](../../crates/worldtools_simulation/src/stages/hydrology.rs):
   fluvial mutation.
4. [`glaciation.rs`](../../crates/worldtools_simulation/src/stages/glaciation.rs):
   ice erosion, rebound, and deposits.
5. `sample_elevation` in
   [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs): atlas
   deformation recombined with local procedural relief.
6. [`tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs): conversion
   to render pages.

### Follow one dependency chain

For example, trace a gold occurrence backward:

```text
resource dominance
  <- localized district + host score
  <- convergence/suture/metamorphism or fluvial placer context
  <- plate motion and paleo boundaries / river flow and erosion
  <- seeded plates + surface elevation + climate runoff
```

Start in `process_scores` in
[`resources.rs`](../../crates/worldtools_simulation/src/stages/resources.rs), then
follow only the indexed fields it reads. This is faster than reading every stage
front to back.

### Tests as executable specification

The most useful invariants are near the code they constrain:

- seed domain separation in
  [`seed.rs`](../../crates/worldtools_world/src/seed.rs);
- cross-tile and parent/child terrain continuity in
  [`terrain.rs`](../../crates/worldtools_world/src/terrain.rs);
- antimeridian sampling in
  [`grid.rs`](../../crates/worldtools_simulation/src/grid.rs);
- bit determinism, typed/packed channel agreement, physical bounds, and
  sub-atlas relief in
  [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs);
- per-stage focused invariants at the bottom of each file under
  [`stages/`](../../crates/worldtools_simulation/src/stages/).

---

# World Data Layers

WorldTools exposes one elevation surface and seven thematic datasets through an
immutable [`WorldSnapshot`](../../crates/worldtools_simulation/src/snapshot.rs).
This chapter is the field-level contract: what is stored, how it is sampled,
what the renderer receives, and what each number does not mean.

For the algorithms that produce these fields, read
[`simulation.md`](simulation.md).

## Three views of the same snapshot

There are three related APIs:

```text
WorldSnapshot
    |
    +-- sample(point) ----------------> WorldSample
    |                                  all typed public fields
    |
    +-- sample_layer(point, layer) ---> [f32; 4]
    |                                  renderer-facing packed channels
    |
    +-- sample_elevation(point) ------> f32 metres
                                       detailed terrain + atlas deformation
```

The type and enum definitions are in
[`layers.rs`](../../crates/worldtools_simulation/src/layers.rs). The actual
sampling code is in
[`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs).

Use `sample()` for inspection, analysis, or tools that need meaning. Use
`sample_layer()` only when implementing a consumer of the four-channel rendering
contract. Packed channels omit many stored fields and sometimes normalize them.

## Sampling rules

Every simulation field is an array aligned to one
[`AtlasGrid`](../../crates/worldtools_simulation/src/grid.rs). At a geographic
point:

- floating-point scalar fields are bilinearly interpolated from four cells;
- categorical enums and plate/terrane IDs use the nearest cell;
- longitude wraps across the antimeridian;
- latitude clamps at the north and south edge rows.

Therefore a typed sample can legitimately combine a category from one nearest
cell with interpolated scalar evidence from surrounding cells. A category
boundary is sharp; scalar boundaries are smooth. Do not interpolate raw enum IDs
yourself.

All normalized scalar signals mentioned below are generally clamped to `[0, 1]`
by their producing stage. They are model indices, not calibrated probabilities,
unless the field description explicitly gives a physical unit.

## Stable layer identity

`WorldDataLayer` uses explicit `u8` discriminants that are also shader IDs.
Changing these values is a renderer contract change.

| ID | Layer | Typed source |
|---:|---|---|
| 0 | Elevation | `WorldSample.elevation_m`, `slope` |
| 1 | Tectonics | `TectonicsSample` |
| 2 | Hydrology | `HydrologySample` |
| 3 | Climate | `ClimateSample` |
| 4 | Soil | `SoilSample` |
| 5 | Vegetation | `VegetationSample` |
| 6 | Geology | `GeologySample` |
| 7 | Resources | `ResourcesSample` |

The UI-to-simulation mapping is kept explicit and tested in
[`apps/worldtools/src/layers.rs`](../../apps/worldtools/src/layers.rs).

## Elevation

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `elevation_m` | metres relative to configured sea level | Detailed procedural base plus bilinearly sampled atlas deformation. |
| `slope` | dimensionless rise/run | Central-difference gradient magnitude on the evolved atlas surface. |

Elevation has three related forms:

- `baseline_elevation_m`: procedural terrain sampled at atlas cell centers
  before tectonic deformation;
- `elevation_m`: the atlas surface after tectonics, fluvial processes, and
  glaciation;
- `sample_elevation(point)`: continuous procedural terrain at the point plus
  the interpolated difference between evolved and baseline atlas surfaces.

`sample_atlas_elevation` omits sub-atlas procedural relief. It exists primarily
for low-resolution fallback rendering.

### Packed RGBA

| Channel | Value |
|---|---|
| R | detailed elevation in metres |
| G | atlas slope |
| B | `0` reserved |
| A | `1` |

### Caveats

Slope is derived from the atlas surface, not the detailed procedural elevation.
Two nearby points in one atlas cell can have different detailed heights but
nearly identical sampled slopes. Sea level is a setting, not necessarily zero.

## Tectonics

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `plate_id` | categorical `u16` | Present nearest plate center. IDs are seed-local and have no cross-world identity. |
| `paleo_plate_id` | categorical `u16` | Nearest reconstructed plate center in the simplified past state. |
| `terrane_id` | categorical `u16` | Combined present/paleo ownership signature. |
| `crust` | `CrustKind` | Oceanic or continental crust classification. |
| `boundary_kind` | `BoundaryKind` | Dominant local boundary regime. |
| `crust_age_myr` | million years | Procedural crust age assigned from plate properties and spreading. |
| `crust_thickness_km` | kilometres | Heuristic thickness from crust type, convergence, and sutures. |
| `boundary` | `[0, 1]` | Proximity/intensity of the present Voronoi plate boundary. |
| `convergence` | `[0, 1]` | Boundary-weighted closing component of relative plate motion. |
| `divergence` | `[0, 1]` | Boundary-weighted opening component. |
| `shear` | `[0, 1]` | Boundary-weighted tangential relative motion. |
| `suture` | `[0, 1]` | Inherited paleo-boundary signal away from current boundaries. |
| `metamorphic_grade` | `[0, 1]` | Index derived mainly from convergence and sutures. |
| `volcanism` | `[0, 1]` | Maximum/combined boundary and hotspot activity. |
| `uplift_m` | metres | Net tectonic construction/deformation applied above the procedural baseline; can include negative buoyancy or spreading relief. |

`BoundaryKind` values are: intraplate, continental collision, subduction arc,
island arc, ocean ridge, continental rift, and transform. `CrustKind` is oceanic
or continental.

### Packed RGBA

| Channel | Value |
|---|---|
| R | nearest `plate_id` converted to `f32` |
| G | `convergence - divergence`, approximately `[-1, 1]` |
| B | `uplift_m` |
| A | `volcanism` |

### Caveats

The packed view discards boundary type, paleo plate, terrane, crust, age,
thickness, shear, suture, and metamorphic grade. Plate ID in R is categorical;
never bilinearly blend or numerically compare it. `uplift_m` records tectonic
construction, not final elevation change after erosion and ice.

## Hydrology and glaciation

Glacial outputs live in `HydrologySample` because they describe water/ice and
surface transport in the renderer-facing layer.

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `runoff` | `[0, 1]` | Local climate-derived land runoff, augmented by bounded meltwater. |
| `river_strength` | `[0, 1]` | Log-normalized accumulated routed flow, threshold-shaped toward major rivers. |
| `wetness` | `[0, 1]` | Composite runoff, river, lake, slope/drainage, and meltwater index. |
| `lake` | `[0, 1]` | Depression depth combined with routed water supply. |
| `erosion_m` | metres | Accumulated fluvial plus glacial erosion history. |
| `sediment_m` | metres-like thickness | Sediment mobilized/deposited by hydrology and glaciation, excluding inherited seafloor cover. |
| `maximum_ice_fraction` | `[0, 1]` | Maximum local/routed ice coverage during the modeled glacial episode. |
| `ice_flux` | `[0, 1]` | Log-normalized accumulated downhill ice-flow index. |
| `glacial_erosion_m` | metres | Glacial contribution to erosion alone. |
| `till_m` | metres | Bounded modeled till deposited at ice margins. |
| `outwash_m` | metres | Bounded meltwater-transported glacial sediment. |
| `isostatic_rebound_m` | metres | Bounded rebound proxy from ice and erosion unloading. |

### Packed RGBA

| Channel | Value |
|---|---|
| R | `river_strength` |
| G | `max(wetness, lake)` |
| B | `sediment_m` |
| A | `-maximum_ice_fraction` when ice fraction is above `0.02`; otherwise `erosion_m` |

The sign of A is an intentional tagged union for the shader: negative means ice,
non-negative means erosion. Code consuming the packed layer must branch on sign
before interpreting magnitude.

### Caveats

River strength is normalized against the maximum accumulation in that world; it
is not discharge in cubic metres per second. Lake is an index, not water depth.
`erosion_m` is a cumulative generated-history measure. The glacial fraction
represents a colder historical maximum, not necessarily present-day permanent
ice.

## Climate

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `zone` | `KoppenZone` | Compact annual-climate category. |
| `temperature_c` | degrees Celsius | Annual mean temperature after coarse circulation and fine lapse-rate correction. |
| `precipitation_mm` | millimetres/year | Annual precipitation after moisture transport and fine orographic adjustment. |
| `seasonality` | `[0, 1]` | Normalized annual temperature-range/continentality signal. |
| `wind_east` | metres/second | Signed eastward annual prevailing wind component. |
| `wind_north` | metres/second | Signed northward annual prevailing wind component. |
| `aridity` | `[0, 1]` | Potential evaporation divided by precipitation plus potential evaporation. |

`KoppenZone` values are ice cap, tundra, arid, temperate, continental, tropical,
and ocean. These are broad labels, not the full set of Koppen-Geiger subtypes.

### Packed RGBA

| Channel | Value |
|---|---|
| R | `temperature_c` |
| G | `precipitation_mm` |
| B | `wind_east` |
| A | `wind_north` |

### Caveats

The packed layer omits zone, seasonality, and aridity. Wind components are
signed physical values, while most nearby atlas fields are normalized indices.
Climate represents a long-term annual state; it has no month, storm, humidity,
or daily variability.

## Soil

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `kind` | `SoilKind` | Dominant surface soil class. |
| `depth_m` | metres, `0` over ocean and about `0.02..6` over land | Modeled retained/weathered soil depth. |
| `fertility` | `[0, 1]` | Composite weathering, organic matter, ash, floodplain, aridity, and waterlogging index. |
| `clay_fraction` | `[0, 1]` | Modeled clay share from weathering and deposits. |
| `organic_fraction` | `[0, 1]` | Modeled retained organic share. |
| `drainage` | `[0, 1]` | High means better drainage; derived from slope, waterlogging, and weathering. |

`SoilKind` values are ocean/marine sediment, bare rock, cryosol, desert,
chernozem, forest, laterite, volcanic, alluvial, peat, and saline.

### Packed RGBA

| Channel | Value |
|---|---|
| R | nearest `SoilKind` discriminant as `f32` |
| G | `depth_m` |
| B | `fertility` |
| A | `organic_fraction` |

### Caveats

The packed view omits clay and drainage. R is a categorical ID, not an ordered
soil-quality value. The ocean category's label is "Marine sediment," while its
soil depth, clay, and drainage outputs are zeroed by the ecology stage.

## Vegetation

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `biome` | `Biome` | Dominant equilibrium biome classification. |
| `canopy_fraction` | `[0, 1]` | Potential woody canopy coverage after growth support. |
| `grass_fraction` | `[0, 1]` | Potential grass coverage after growth support. |
| `biomass` | `[0, 1]` | Composite canopy, grass, and organic-matter index. |
| `fire_frequency` | `[0, 1]` | Relative fire-conducive frequency from fuels, heat, aridity, seasonality, and waterlogging. |

Biome values are ocean, ice, tundra, boreal forest, temperate forest,
temperate grassland, Mediterranean scrub, desert, savanna, tropical seasonal
forest, tropical rainforest, alpine, and wetland.

### Packed RGBA

| Channel | Value |
|---|---|
| R | nearest `Biome` discriminant as `f32` |
| G | `canopy_fraction` |
| B | `grass_fraction` |
| A | `biomass` |

### Caveats

Fire frequency is omitted from the packed view. Canopy and grass are potential
fractions and need not sum to one. The stage models no species, succession,
land use, disturbance history, or vegetation-to-climate feedback.

## Geology

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `lithology` | `Lithology` | Dominant exposed surface material/bedrock. |
| `rock_age_myr` | million years | Inherited procedural crust/rock age. |
| `sediment_m` | metres-like thickness | Hydrologic sediment plus a fraction of erosion, bounded by the geology stage. |
| `volcanic_ash_m` | metres-like thickness | Original plus wind-dispersed and rain-washed ash, bounded by the geology stage. |
| `weathering` | `[0, 1]` | Warmth/moisture/slope weathering index with a small ash contribution. |

Lithology values are oceanic basalt, felsic craton, sedimentary rock, volcanic
arc, plutonic rock, metamorphic rock, carbonate platform, and unconsolidated
sediment.

### Packed RGBA

| Channel | Value |
|---|---|
| R | nearest `Lithology` discriminant as `f32` |
| G | `rock_age_myr` |
| B | `sediment_m` |
| A | `volcanic_ash_m` |

### Caveats

Weathering is omitted from the packed view. Lithology is one surface category,
not a vertical stack. `sediment_m` here differs from hydrology sediment: geology
adds a fraction of erosion before clamping. Ash transport is a fixed 14-step
advection pass, not a dated eruption record.

## Resources

### Typed fields

| Field | Unit/range | Meaning |
|---|---|---|
| `dominant` | `ResourceDeposit` | Richest localized occurrence crossing its model threshold, or none. |
| `richness` | `[0, 1]` | Normalized occurrence strength within the chosen deposit model. |
| `depth_m` | metres | Procedural depth estimate from deposit type and local context. |
| `confidence` | `[0, 1]` | Composite model-evidence confidence for the selected occurrence. |
| `metallic` | `[0, 1]` | Best metallic group prospectivity, including sub-threshold signals. |
| `energy` | `[0, 1]` | Best energy group prospectivity. |
| `industrial` | `[0, 1]` | Best industrial group prospectivity. |

Deposit values are none, banded iron, bauxite, porphyry copper, volcanogenic
massive sulfide, nickel sulfide, gold, gemstones, coal, peat, petroleum, natural
gas, rock salt, clay, phosphate, and nitrate.

### Packed RGBA

| Channel | Value |
|---|---|
| R | nearest `ResourceDeposit` discriminant as `f32` |
| G | `richness` |
| B | `clamp(depth_m / 6000, 0, 1)` |
| A | `confidence` |

Depth is the only packed channel whose scale differs from its typed value.
Multiply B by 6000 metres only when it was not clamped at one; values at one
mean 6000 metres or deeper within the packed contract.

### Caveats

The packed view omits group prospectivity. R is categorical. Confidence is
confidence in the procedural evidence, not a statistical survey probability.
No field represents tonnage, ore grade, extraction cost, ownership, accessibility,
or market value.

## Typed categories and raw storage

Inside `WorldSnapshot`, categorical atlas arrays are stored compactly as `u8`
or `u16`. Public sampling converts bytes to enums using tolerant `from_byte`
functions. Unknown byte values generally fall back to the default/ocean/none
variant. That protects callers from invalid enum construction but can hide
corrupt internal data if raw arrays are ever loaded from an external format.

The enum discriminants are part of the shader contract. Treat reordering or
inserting variants in the middle as a data-format change. Append deliberately,
update conversion functions and `COUNT`, then update shader palettes and tests.

## Render-page representation

[`MapTileData`](../../crates/worldtools_render/src/tile_data.rs) resamples one
active layer from the snapshot into map tiles. Every page owns:

- an `R32Float` elevation texture;
- an `Rgba32Float` texture containing the active layer's packed channels;
- minimum and maximum elevation for the page;
- one-sample apron around the page for continuous filtering and derivatives.

The elevation texture remains present even when the thematic layer is active,
so the renderer can shade thematic data over relief. Changing active layers
invalidates tile pages because each page contains only one thematic RGBA dataset,
not all seven at once. The streamer's revision and layer checks reject stale
asynchronous results. See
[`streaming.rs`](../../crates/worldtools_render/src/streaming.rs).

Display modes are presentation, not additional generated datasets. Physical,
relief, slope, and contour modes can derive visual effects from elevation and
screen-space derivatives. The mode and styling uniforms are defined in
[`display.rs`](../../crates/worldtools_render/src/display.rs); shader decoding is
in [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl).

## Probe representation

The application probe calls `sample_world`, then formats typed values with
units and enum labels. Its display contract is a useful quick check for field
meaning:

- physical quantities receive `m`, `km`, `Myr`, `C`, `mm/yr`, or `m/s`-derived
  formatting;
- normalized indices become percentages;
- categories use enum labels.

Read [`probe/format.rs`](../../apps/worldtools/src/interaction/probe/format.rs)
when a UI label seems to disagree with a field name. It exposes more information
than the four packed render channels.

## Consumer checklist

Before adding analysis, export, or a new shader:

1. Decide whether you need typed fields or only packed RGBA. Prefer typed fields
   outside rendering.
2. Check whether a field is physical, normalized, categorical, or a tagged
   packed value.
3. Use nearest sampling for categorical data and bilinear sampling for scalar
   data.
4. Use configured sea level rather than assuming zero.
5. Distinguish atlas elevation from composed detailed elevation.
6. Treat plate, terrane, biome, soil, lithology, and deposit numbers as IDs, not
   magnitudes.
7. Account for omitted packed fields before drawing scientific conclusions from
   a rendered layer.
8. Preserve explicit enum discriminants and packed channel order when changing
   contracts.
9. Add an agreement test like `typed_and_direct_layer_channels_share_one_contract`
   in [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs) when
   modifying packed output.

## Where to read next

- Generation and coupling: [`simulation.md`](simulation.md)
- Public types: [`layers.rs`](../../crates/worldtools_simulation/src/layers.rs)
- Storage and sampling: [`snapshot.rs`](../../crates/worldtools_simulation/src/snapshot.rs)
- Grid topology: [`grid.rs`](../../crates/worldtools_simulation/src/grid.rs)
- Render packing: [`tile_data.rs`](../../crates/worldtools_render/src/tile_data.rs)
- Shader interpretation: [`worldtools_tile.wgsl`](../../crates/worldtools_render/src/worldtools_tile.wgsl)
- Human-readable probes: [`probe/format.rs`](../../apps/worldtools/src/interaction/probe/format.rs)

---

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

---

# Debugging and Diagnostics

WorldTools debugging is evidence-first. Reproduce with explicit inputs, retain artifacts, narrow the failing boundary, then make one coherent change. This chapter covers the repository's built-in tooling and does not require internet access.

## First response

Use this order unless the symptom clearly demands a narrower path:

1. Run `cargo xtask doctor` and retain the capability report.
2. Select or create a deterministic TOML case under [`.debug/cases`](../../.debug/cases).
3. Reproduce before changing source with `cargo xtask repro <case>` or `cargo xtask capture <case>`.
4. Record no more than three ranked hypotheses. For each, name an observation that would falsify it.
5. Run the cheapest experiment that separates those hypotheses.
6. Capture in-app diagnostics before mutating live state.
7. Patch only the demonstrated cause.
8. Repeat the original case, format, and run `cargo xtask check quick`; broaden checks in proportion to the change.

Do not run concurrent Cargo commands against the same `target` directory.

## The two artifact families

WorldTools keeps application evidence separate from development-harness evidence:

```text
.runtime/diagnostics/       one running app's logs, panic reports, snapshots, audits
.debug/runs/<run-id>/       one bounded xtask repro, capture, or debugger-script run
```

This distinction matters. A diagnostic snapshot describes live ECS/resources at one point in an interactive session. An xtask result describes a controlled external command and whether each attempt met its expected exit behavior.

Both roots are excluded from source-control work. Preserve the relevant directory path in a bug report rather than pasting only a final error line.

## Host capability report

[`cargo xtask doctor`](../../xtask/src/doctor.rs) prints JSON describing the platform, Rust host, repository revision/dirty state, and available development tools. To retain a stable copy:

```powershell
cargo xtask doctor --output .debug/doctor.json
```

The probe includes Git, Rust/Cargo, Codex, LLDB and LLDB integrations, CDB, GDB, rr, perf, nextest, Miri, cargo-audit, cargo-deny, Samply, cargo-flamegraph, Tracy, and RenderDoc. Availability is a capability report, not a requirement that every machine install every tool.

Use the report to choose an investigation lane. For example, do not plan a CDB session until `doctor` confirms CDB; do not claim a GPU capture was taken merely because RenderDoc is named in the workflow.

## Deterministic cases

A case is a strict TOML document parsed by [`xtask/src/case.rs`](../../xtask/src/case.rs). Supported fields are:

```toml
name = "short human-readable label"       # optional
command = ["program", "arg1", "arg2"]    # required
working_directory = "relative/path"       # optional, workspace by default
expected_exit = 0                          # default 0
timeout_seconds = 180                      # default 30, must be nonzero
repeat = 2                                 # default 1, must be nonzero
seed = 91842                               # optional

[env]
RUST_LOG = "worldtools=debug"

[debug]
program = "target/debug/worldtools.exe"
args = []
breakpoints = ["worldtools::main", "src/file.rs:42"]
```

When `seed` is present, xtask supplies `WORLDTOOLS_SEED` unless the case's environment already defines it. It also defaults `RUST_BACKTRACE` to `full`. `repeat` is part of the contract: a flaky or timing-sensitive failure is not resolved by one lucky passing attempt.

The checkout includes:

- [`terrain-smoke.toml`](../../.debug/cases/terrain-smoke.toml): runs `worldtools-lab verify` twice at seed 91842 and includes a native-debug target.
- [`world-history.toml`](../../.debug/cases/world-history.toml): runs simulation library tests twice at the same seed.

Create a new case when the command, seed, timeout, environment, or expected result differs materially. Do not keep changing one case until it no longer represents the original failure.

## Repro versus capture

Both commands use the same bounded runner in [`xtask/src/reproduce.rs`](../../xtask/src/reproduce.rs):

```powershell
cargo xtask repro terrain-smoke
cargo xtask capture terrain-smoke
```

`repro` retains command evidence. `capture` additionally writes the current `doctor.json` into the run directory. A generated directory under `.debug/runs` contains:

| Artifact | Meaning |
|---|---|
| `meta.json` | Case path, command, working directory, expected exit, timeout, repeat, seed, environment key names, start time. |
| `result.json` | Overall success and structured result for each attempt. |
| `attempt-NN-stdout.log` | Exact stdout for one attempt. |
| `attempt-NN-stderr.log` | Exact stderr for one attempt. |
| `stdout.log`, `stderr.log` | Combined logs with attempt separators. |
| `doctor.json` | Host/tool/repository report; present for `capture`. |

The command returns failure if any attempt times out or exits differently from `expected_exit`. `result.json` is updated after each attempt, so partial evidence survives an interrupted multi-attempt run.

## In-app diagnostics

Diagnostics are installed before Bevy's default plugins in [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs) and implemented by [`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs). The default directory is:

```text
<working-directory>/.runtime/diagnostics
```

Override it with `WORLDTOOLS_LOG_DIR`. Override the application target filter with `WORLDTOOLS_LOG`; the default enables debug events for WorldTools app/render/UI targets and info for world generation. Files are written without ANSI color and include source location, target, thread, and span-close events.

Example PowerShell launch with a dedicated evidence directory:

```powershell
$env:WORLDTOOLS_LOG_DIR = ".debug/runs/manual-map-session"
$env:WORLDTOOLS_LOG = "worldtools=debug,worldtools_render=trace"
cargo run -p worldtools
```

The directory can contain:

- `worldtools.log.YYYY-MM-DD`: rolling daily trace log.
- `panic-<timestamp>-<pid>.txt`: build/platform/process metadata, panic location and message, diagnostic environment, and forced backtrace.
- `snapshot-<timestamp>.json`: live application, system, document, simulation, view, tile, renderer, layer, and recent-event state.
- `terrain-audit-<timestamp>.json`: deterministic terrain distribution, seam, parent-child LOD, finite-value, and repeatability results.

Snapshot and audit JSON is written through a temporary file and atomically renamed, so a final filename represents a complete serialization.

## Diagnostics window

Press F12 to open the native diagnostics window implemented in [`debug_window.rs`](../../crates/worldtools_ui/src/shell/debug_window.rs). It has five tabs:

- **Summary**: FPS, frame time, frame number, entities, process CPU/RAM, rendered/degraded page counts, generation activity, snapshot, and terrain-audit actions.
- **Streaming**: LOD, visible/resident/in-flight/ready counts, request/completion/discard/invalidation counters, generation timings, cache/queue limits, render states, overlays, freeze, and cache flush.
- **Viewport**: geographic center, vertical span, logical/physical dimensions, pixels per point, LOD, and ground resolution.
- **Layers**: native layer availability and current selection.
- **Events**: bounded, filterable in-memory trace events with dropped-event count.

The event path is deliberately non-blocking. The tracing layer has a 2,048-event channel; the app drains at most 256 per frame into a 512-entry UI log. Drops are counted at both boundaries. A high drop count means the visible event list is incomplete; use the rolling file log as the retained source.

## Streaming diagnostics

The most useful renderer controls map directly to [`RenderDebugSettings`](../../crates/worldtools_render/src/debug.rs):

- **Tile borders** shows exact projected page edges.
- **LOD tint** colors desired LODs with a stable palette.
- **Residency tint** distinguishes exact, ancestor fallback, and stale pages.
- **Trace lifecycle** emits tile planning, request, generation, acceptance/discard, and surface-state events.
- **Freeze tile streaming** stops new requests while allowing in-flight work to complete.
- **Flush tile cache** invalidates resident and in-flight IDs so revisions reject late results and visible pages are requested again.

Interpret counters together:

- `visible` is visual placements, including multiple wrapped placements of one canonical page.
- `resident_visible` includes exact pages and usable ancestors.
- `resident_total` is the CPU cache entry count; capacity is 128.
- `in_flight` is active jobs; scheduler limit is 8.
- `ready_results` is the completed-result channel depth; the app consumes at most 8 per frame.
- `discarded` should rise when layer changes or invalidations make late work stale. A rising value alone is not a correctness failure.
- `fallback`, `stale`, and `missing` describe what was submitted to the GPU, not merely CPU cache contents.

Capture a snapshot before toggling freeze or flushing the cache if the current state is important.

## Diagnostic snapshots and terrain audits

**Snapshot** emits schema `worldtools.diagnostic-snapshot.v2` from [`snapshot.rs`](../../apps/worldtools/src/debug_tools/snapshot.rs). It includes the world fingerprint and simulation settings, view, UI state, telemetry, stream/render stats, sorted resident and in-flight IDs with revisions, layer capabilities, system info, and up to 200 recent events.

Use the fingerprint to compare generated-world identity across sessions. Use `world_epoch`, tile revision, source layer, and recent events to distinguish an old result from the current world.

**Terrain audit** runs asynchronously from [`audit.rs`](../../apps/worldtools/src/debug_tools/audit.rs). It checks all six root cube faces for finite terrain distribution, a known cross-face seam, a parent-child LOD relationship, and deterministic regeneration. Its strict pass condition requires zero-bit error for the tested seam and LOD pair and no non-finite elevation samples.

The audit diagnoses world-generation consistency. It does not prove that viewport placement, GPU upload, or shader presentation is correct; use render overlays and screenshots for those boundaries.

## Live Bevy Remote inspection

The optional endpoint is implemented in [`live_remote.rs`](../../apps/worldtools/src/live_remote.rs) and compiled only with the `live-debug` feature:

```powershell
$env:WORLDTOOLS_BRP = "1"
cargo run -p worldtools --features live-debug
```

It binds to IPv4 loopback, is disabled unless explicitly enabled, and refuses to start in a release build. The custom read-only `worldtools.status` method returns document, view, simulation, streaming, renderer, performance, and debug state.

Mutation methods are rejected by default. Writable access requires a second explicit opt-in before launch:

```powershell
$env:WORLDTOOLS_BRP_ALLOW_WRITE = "1"
```

Start with status and read-only ECS queries. Take a diagnostic snapshot first. Resource/component mutation, events, spawning, despawning, and reparenting change the evidence and should be used only with explicit intent.

## Native debugger scripts

Generate a noninteractive LLDB or CDB command file from a case:

```powershell
cargo xtask debug-script terrain-smoke --backend lldb
cargo xtask debug-script terrain-smoke --backend cdb
cargo xtask debug-script terrain-smoke --backend cdb --output .debug/session.cdb
```

Add `--run` to start the selected debugger after generation. The implementation is in [`debug_script.rs`](../../xtask/src/debug_script.rs). LLDB scripts configure the working directory, target arguments, environment, source/symbol breakpoints, then capture all thread backtraces and frame variables. CDB scripts break on access violations, apply symbol breakpoints, then capture all stacks and locals.

Attach a native debugger only after narrowing the smallest useful breakpoint or watchpoint. At a stop, inspect the triggering thread, full stack, arguments, relevant locals, and the violated invariant. For high-frequency systems, prefer conditional breakpoints or tracepoints.

## Symptom-to-tool guide

| Symptom | First evidence | Best next tool | What would discriminate the cause |
|---|---|---|---|
| App exits or panics | Panic report and rolling log | `cargo xtask capture <case>`; LLDB/CDB script after narrowing | Repeated stack/location with the same seed versus host-specific failure. |
| Generated world differs between runs | Snapshot fingerprints and settings | `world-history` case; terrain audit | Same inputs with different fingerprints or audit determinism false. |
| Crack at a tile boundary | Screenshot plus tile borders | Residency/LOD tints; terrain audit; shader/source inspection | CPU seam audit zero but visible crack implicates sampling, apron, transform, or GPU presentation. |
| Flashing tiles after layer/world change | Snapshot before change, trace lifecycle | Freeze streaming; inspect revisions, layer, `world_epoch`, discarded count | Accepted result with mismatched epoch/revision/layer versus correctly discarded late work. |
| Blank holes while moving | Streaming and renderer telemetry | Residency tint; trace lifecycle; snapshot | `missing > 0` with no roots/in-flight points to scheduling; resident pages with holes points to surface/GPU state. |
| Permanently blurry region | LOD tint and exact/fallback counts | Snapshot tile IDs; flush cache once | Desired LOD never requested/accepted versus exact page resident but sampled incorrectly. |
| Map offset or wrong size | Viewport tab logical/physical sizes | Inspect UI `MapViewport`, bridge, render `MapViewport` | Difference caused by egui zoom/window scale conversion rather than projection. |
| Map responds through UI | `input_blocked`, hovered state | Inspect egui viewport ownership and navigation settings | Map starts a gesture while egui reports pointer ownership. |
| Wrong colors for a layer | Active layer/mode and snapshot layer state | Inspect bridge, channel contract, shader mode | Correct layer page with wrong shader interpretation versus stale previous-layer page. |
| Frame-time spike during movement | Frame telemetry and generation timings | Optimized stable workload with Samply/ETW/perf/Tracy | CPU tile generation, result upload, ECS/surface churn, or GPU work correlates with spike. |
| GPU-only corruption or validation error | Screenshot, wgpu log, tile state snapshot | RenderDoc capture | Correct ECS/material handles but incorrect texture/bind-group/draw state in captured frame. |
| Test/compiler failure | Full stderr and exact command | Focused test or nextest after `doctor` | Minimal package/filter reproduces independently of unrelated targets. |
| Suspected undefined behavior | Small deterministic case | Miri or applicable sanitizer | Failure under instrumentation at the same operation. |

For a visual defect, correlate at least four layers of evidence: viewport/camera state, visible/resident tile state, screenshot, and wgpu/RenderDoc evidence. A screenshot alone cannot distinguish bad source data from bad placement or shader interpretation.

For a performance defect, reproduce in an optimized build with a fixed seed and stable interaction. Source shape is not profiling evidence.

## Verification commands

The sequential verification harness is implemented in [`xtask/src/check.rs`](../../xtask/src/check.rs):

```powershell
cargo fmt --all
cargo xtask check quick
cargo xtask check full
cargo xtask check full --miri
```

`quick` checks formatting, the whole workspace/all targets, Clippy with warnings denied, and workspace library tests. `full` replaces library-only tests with all workspace targets. Miri is accepted only with `full` and requires the appropriate nightly/tool component.

For a narrow Rust change, run a focused package test first when available:

```powershell
cargo nextest run -p <package> <filter>
```

Then run the original reproduction at least as many times as its case specifies. In the final report, name the failing-before command, passing-after command, evidence directory, demonstrated root cause, and remaining uncertainty.

## Source map

- App assembly and diagnostics: [`main.rs`](../../apps/worldtools/src/main.rs), [`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs), [`debug_tools`](../../apps/worldtools/src/debug_tools/mod.rs)
- Renderer telemetry and controls: [`streaming.rs`](../../crates/worldtools_render/src/streaming.rs), [`debug.rs`](../../crates/worldtools_render/src/debug.rs), [`tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs)
- UI diagnostics model and window: [`model/debug.rs`](../../crates/worldtools_ui/src/model/debug.rs), [`shell/debug_window.rs`](../../crates/worldtools_ui/src/shell/debug_window.rs)
- Deterministic harness: [`xtask/src/cli.rs`](../../xtask/src/cli.rs), [`case.rs`](../../xtask/src/case.rs), [`reproduce.rs`](../../xtask/src/reproduce.rs), [`artifact.rs`](../../xtask/src/artifact.rs)
- Rendering architecture: [Rendering, UI, and Runtime Integration](rendering-ui.md)

---

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

---

# Glossary

## Application terms

**WorldTools**  
The native application and the workspace as a whole. The binary package is `worldtools`.

**World snapshot**  
An immutable, seed-derived global simulation result. [`WorldSnapshot`](../../crates/worldtools_simulation/src/snapshot.rs) is the authoritative source sampled by rendering and inspection.

**World epoch**  
A renderer-side generation identity. Replacing the snapshot changes the epoch so pages and asynchronous results from an older world cannot be presented as current.

**Revision**  
An identity attached to generated or diagnostic state. Exact meaning depends on the owning subsystem; it is used to reject stale asynchronous results and correlate evidence.

## Coordinates and tiling

**GeoPoint**  
A latitude/longitude position with canonical geographic behavior, defined in [`geo.rs`](../../crates/worldtools_world/src/geo.rs).

**Cube tile**  
A spherical terrain tile in `worldtools_world`. Cube-face coordinates support deterministic base terrain and exact shared samples between neighboring and parent/child tiles.

**Map tile**  
A two-dimensional equirectangular presentation page in `worldtools_render`. It is not the same type or topology as a cube tile.

**Desired tile**  
The page resolution selected for the current viewport.

**Source tile**  
The resident page actually used for a desired placement. It may be an ancestor fallback while the exact page is generated.

**Placement**  
A visible instance of a canonical map tile. Its `unwrapped_x` preserves which repeated world copy is on screen.

**Canonical tile ID**  
The wrapped identity used for generation and caching. Multiple horizontal placements can reference the same canonical data.

**LOD**  
Level of detail. Higher levels contain more, smaller pages. LOD is selected from viewport resolution and capped by the supported source-resolution limit.

**Apron / halo**  
Samples stored around a tile's visible cells. Shader filters and neighborhood operations use the halo without clamping at every tile edge.

**Fallback**  
A coarser resident ancestor page temporarily used when an exact page is unavailable.

## Simulation

**AtlasGrid**  
The global equirectangular storage used by coupled simulation fields. Continuous and categorical sampling use different rules.

**Stage**  
A deterministic transformation in world-history generation. Stage order encodes dependencies and therefore model semantics.

**Coupling**  
One natural process influencing another: tectonics affects elevation and geology; elevation affects hydrology and climate; climate affects glaciation, ecology, and soils; geology and hydrology affect resources.

**Domain-separated randomness**  
Deriving independent pseudo-random keys from an explicit root seed and subsystem identity. Adding one random decision in geology should not shift every climate result.

**Fine relief**  
Deterministic elevation detail evaluated below the global simulation atlas cell scale. It must remain conditioned by large-scale terrain rather than acting as unrelated decorative noise.

**Layer channels**  
The four `f32` values packed into a tile's RGBA32 data texture for one active `WorldDataLayer`. Channel meanings are layer-specific.

**Categorical field**  
A field whose values are identities or classes, such as plate, biome, soil, or lithology. It must not be interpolated as if IDs were measurements.

**Continuous field**  
A scalar or vector measurement, such as elevation, temperature, precipitation, or discharge. It may be smoothly sampled if bounds and physical meaning are preserved.

## Rendering

**MapView**  
Camera-like normalized center and vertical span for the flat map. Longitude remains unwrapped to support endless horizontal panning.

**Tile streamer**  
The asynchronous scheduler and page cache that turns visible demand into `MapTileData`.

**Residency**  
Whether the GPU/CPU page needed for a visible placement is exact, fallback, stale, or missing.

**Physical view**  
Natural terrain and bathymetry presentation derived from elevation, slope, latitude, and relief lighting.

**Layer presentation**  
Visual controls that do not change generated data: opacity, relief, shadows, detail, categorical borders, sun direction, ambient occlusion, labels, and legend.

**Multiscale relief**  
Lighting that combines fine and broad height derivatives, concavity, and a short horizon test to imply depth without a terrain mesh.

**World-space decoration**  
A pattern anchored to canonical geographic coordinates. Unlike screen-space patterns, it does not slide when the camera pans or reset at tile boundaries.

## UI and runtime

**EditorUiState**  
Semantic user intent: active tool, layer, map view, and analysis state. It deliberately contains no renderer implementation details.

**Viewport bridge**  
Application integration that translates UI resources into renderer viewport, navigation, active layer, and display settings.

**Probe**  
A pinned geographic sample formatted for the active layer.

**BRP**  
Bevy Remote Protocol. In WorldTools it is development-only, loopback-bound, and disabled unless explicitly requested.

**Evidence bundle**  
A deterministic `.debug/runs/...` directory produced by `cargo xtask capture`, containing metadata, stdout/stderr, results, and relevant diagnostic artifacts.

**Quick lane**  
`cargo xtask check quick`: formatting, type checking, strict Clippy, and focused workspace library tests.

**Full lane**  
`cargo xtask check full`: broader validation appropriate before high-risk integration or release work.

