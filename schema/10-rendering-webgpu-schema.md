# Rendering And WebGPU Schema

Rendering and GPU diagnostics have separate responsibilities. Canvas 2D is the authoritative map renderer; WebGPU is an optional validation accelerator. The app remains fully usable when WebGPU is absent.

## Canvas Renderer

`src/gpu/renderer.ts` coordinates focused modules:

- `mapView.ts` owns projection-frame, pan, zoom, and coordinate conversion math;
- `rasterizer.ts` builds and caches one semantic raster per overlay;
- `cartography.ts` draws graticule, coastlines, rivers, plate contacts, selections, and frame;
- `renderer.ts` owns world/view/overlay state and PNG export.

Categorical overlays use nearest-style scaling so class colors are not interpolated. Continuous terrain uses hillshade and smoothed color scales. Latitude row order is explicitly inverted for north-up display. Region hit testing uses the same map frame and grid indexing as rendering.

## Overlay Contract

The data-driven overlay catalog defines ID, label, units, source stage, layer reference, category/continuous behavior, value accessor, and palette. Current overlays cover terrain, plates, contacts, elevation, slope, hydrology, erosion, sediment, temperature, precipitation, biome, resources, and suitability.

Every overlay exposes a legend, units, source-stage number, min/max where available, and runtime provenance. Selection and comparison markers survive overlay changes.

## WebGPU Diagnostic

When a device is available, the elevation diagnostic uploads one eight-float record per region:

```text
surfaceElevation, minElevation, maxElevation, seaLevel,
normalizedElevation, landFlag, finiteFlag, reserved
```

A 64-thread WGSL compute pass writes normalization, land, and finite flags. Full readback verifies cell count, NaN count, land count, and maximum normalization error. Buffers are destroyed after each diagnostic. Device absence or loss is visible in runtime status and never masquerades as a GPU-backed simulation.

## Acceptance

- The map is non-empty after a world is installed and resizes at bounded device-pixel ratio.
- Longitude seams, north/south orientation, categorical colors, and hit testing agree.
- Pan/zoom remain constrained and do not shift surrounding layout.
- Optional GPU output is finite and normalization error is below tolerance.
- PNG export reflects the current map and overlay.
