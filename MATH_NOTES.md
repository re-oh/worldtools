# Project Bombo Model Notes

This document specifies the model implemented in `src/simulation`. Values are designed for deterministic, causal plausibility and inspectability. They are not calibrated scientific predictions.

## Determinism

Generation is a pure function of normalized `WorldParameters`, schema version `0.2.0`, and deterministic hash streams. Noise is sampled from each cell's unit-sphere vector, so the longitude seam does not introduce a discontinuity. Timestamps and measured durations are provenance only and are excluded from replay identity.

## Spherical Region Graph

For latitude band `i` out of `N`, the cell center is sampled uniformly in sine latitude:

```text
sin(latitude_i) = -1 + 2 * (i + 0.5) / N
longitude_j     = -180 + 360 * (j + 0.5) / M
cell_area       = 4 * pi * radius^2 / (N * M)
```

Each cell connects to the available cells in the surrounding 3 by 3 index neighborhood. Longitude wraps; latitude does not cross a pole. Interior cells have eight neighbors and polar-row cells have five. Adjacency is sorted, symmetric, and deterministic.

This representation equalizes nominal cell area and supports fast array lookup, but graph edge lengths and polar shapes are not uniform.

## Plate Construction

Plate origins use farthest-candidate sampling: each new origin chooses the best of 48 deterministic candidates by minimum angular separation from existing origins. A min-heap then grows all plate frontiers together. The step cost combines:

- angular distance from the plate origin;
- directional anisotropy;
- deterministic local variation;
- a small continental latitude bias.

Every claimed cell is reached from an already claimed neighbor, so every plate remains connected. Crust type is derived from each plate's continental/oceanic/mixed bias and seam-safe spherical noise.

## Plate Motion And Boundaries

Each plate receives a unit Euler-pole vector and angular speed in degrees per million years. At unit-sphere point `p`, the surface velocity is:

```text
omega = radians(angular_velocity_degrees_per_Ma)
v_cm_per_year = cross(normalize(euler_pole), p) * omega * radius_km * 0.1
```

For neighboring cells owned by different plates, relative velocity is projected onto the local tangent plane. The component across the contact is signed convergence in centimeters per year; the absolute along-contact component is shear. Thresholds classify convergent, divergent, transform, or oblique contacts.

Boundary potentials are normalized heuristics. Uplift depends on convergence and crust pairing, trench potential requires oceanic crust, and volcanic/earthquake potentials combine convergence, divergence, and shear. Maximum influence diffuses outward for two to four graph rings with exponential decay so terrain forms belts instead of single-cell ridges.

## Base Terrain

Base elevation combines crust-specific reference elevation, plate age, four-octave macro noise, and three-octave detail noise. Tectonic elevation adds uplift and volcanism and subtracts trench influence using crust-specific scales. Initial surfaces are clamped to `-8800..8800 m`.

The represented elevation terms are kept separate:

```text
surface = base + tectonic - erosion + deposition
water_depth = max(0, sea_level - surface)
```

This explicit decomposition is persisted per region and exposed as layers, rather than compressing terrain into an opaque color or single untraceable height.

## Erosion And Hydrology

Thermal erosion runs the configured number of iterations. A cell transfers material toward its lowest neighbor when the local drop exceeds a distance-aware stable drop. Per-step transfer is bounded, tracked as erosion at the source and deposition at the target, and applied through a separate delta buffer.

Drainage uses a priority-flood seeded by ocean cells. If a generated world contains no ocean, the global minimum is the seed. The flood creates a depression-resolved elevation and a deterministic downstream pointer for every cell. Basins are resolved by path compression, and validation rejects non-neighbor routing or cycles.

Runoff is first estimated from latitude belts, terrain noise, and slope. Area-weighted discharge accumulates downstream in descending filled-elevation order. River incision scales with normalized discharge and square-root slope and is capped at 260 meters. Shallow marine cells receive bounded deposition. Drainage is recomputed after incision and deposition.

The stored fields are `erosionDepthMeters`, `depositionDepthMeters`, `sedimentDepthMeters`, `runoffMmYear`, `riverDischargeKm3Year`, normalized `flowAccumulation`, `downstreamRegionId`, and `drainageBasinId`.

## Climate

Ocean distance is the shortest great-circle path through the region graph, calculated with Dijkstra's algorithm from all ocean cells. Mean temperature combines a latitude curve, a `6.2 C/km` lapse rate, ocean moderation, and low-amplitude spherical noise. Annual range increases with continentality and latitude.

Precipitation combines equatorial and mid-latitude wet belts, a subtropical dry belt, polar drying, exponential ocean moisture supply, deterministic variation, and one-cell windward/leeward terrain sampling. Prevailing wind reverses by latitude band. Aridity compares precipitation with a temperature-derived potential evapotranspiration proxy.

After climate is known, runoff and discharge are recalculated with climate-dependent infiltration and accumulated through the fixed drainage graph.

## Classification And Suitability

Ten biome classes are selected from water state, temperature, aridity, and rainfall. Resource IDs are deterministic signals derived from volcanism, convergence, discharge, elevation, biome, and coastal position.

Suitability is always evaluated for one explicit use case. Physical, water, soil, resource, and access scores are weighted by that use case; tectonic and slope hazards are subtracted; contextual modifiers cover requirements such as ports or mineral access. Scores are clamped to `0..100`, and every result stores human-readable explanation references.

## Validation

The test and runtime validation layers check:

- schema-valid deterministic replay;
- equal nominal area and symmetric adjacency;
- connected plates and complete boundary classification;
- finite, bounded terrain and climate fields;
- non-negative water, erosion, deposition, runoff, and discharge;
- neighbor-only acyclic drainage;
- valid biome and suitability outputs;
- archive checksum round trips and tamper rejection;
- explicit WebGPU buffer layout and finite diagnostic output.

## Limitations

The highest-value future model work is a lower-distortion spherical mesh, time-integrated plate history, explicit lake/endorheic handling, conservative hydraulic sediment transport, multi-cell atmospheric moisture advection, and calibrated realism metrics for slope, basin geometry, river density, and mountain continuity.
