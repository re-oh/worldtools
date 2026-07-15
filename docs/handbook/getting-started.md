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

