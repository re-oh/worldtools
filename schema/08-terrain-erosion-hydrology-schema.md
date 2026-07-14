# Terrain, Erosion, And Hydrology Schema

The terrain pipeline is implemented on the region graph by `src/simulation/terrain.ts`. It keeps source terms separate so a final height can be explained rather than stored as an opaque compressed value.

## Terrain Representation

| Field | Units | Meaning |
| --- | --- | --- |
| `baseElevationMeters` | m | crust, plate age, macro/detail spherical noise |
| `tectonicElevationMeters` | m | uplift + volcanism - trench influence |
| `surfaceElevationMeters` | m | final clamped surface |
| `erosionDepthMeters` | m | cumulative removed material |
| `depositionDepthMeters` | m | cumulative deposited material |
| `sedimentDepthMeters` | m | current deposition proxy |
| `slope` | rise/run | maximum local great-circle gradient |
| `waterDepthMeters` | m | `max(0, seaLevel - surface)` |

Continental, transitional, and oceanic crust use different base elevation, age, uplift, trench, and volcanic scales. Macro and detail noise sample unit-sphere vectors and remain continuous across the longitude seam. Initial elevation is clamped to `-8800..8800 m`; post-process elevation is clamped to `-9000..9000 m`.

## Thermal Erosion

The configured iteration count runs through a separate delta buffer. Each cell finds its steepest downhill neighbor using great-circle edge distance. Material moves only when the drop exceeds a distance-aware stable drop, and each transfer is bounded. Source erosion and target deposition are accumulated explicitly.

## Drainage

Priority flood begins at every ocean cell, or the global minimum if the world has no ocean. The algorithm derives a depression-resolved filled elevation and one downstream pointer per cell. Downstream accumulation is ordered by filled elevation; basins are assigned with path compression.

Initial runoff uses latitude belts, terrain variation, and slope. Discharge converts runoff and cell area to km3/year and accumulates downstream. `flowAccumulation` is logarithmically normalized against maximum land discharge.

## Incision And Deposition

River incision depends on normalized discharge, square-root slope, and erosion iterations and is capped at 260 m. Shallow marine cells receive up to 34 m of deposition. Drainage is recalculated after these changes. Climate later refines runoff and discharge without changing the downstream graph.

## Invariants

- Depth, runoff, and discharge fields are finite and non-negative.
- Downstream IDs exist and are either self-sinks or direct neighbors.
- Following downstream pointers terminates without a cycle.
- Basin IDs exist and identify the resolved sink.
- Surface decomposition remains consistent within floating-point tolerance.

This is a regional approximation. It does not model transient water depth, conserved suspended sediment, stream order, groundwater, glaciers, or persistent endorheic lakes.
