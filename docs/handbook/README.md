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
