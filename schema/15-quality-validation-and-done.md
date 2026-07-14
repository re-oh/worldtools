# Quality, Validation, And Release Gates

## Release `0.2.0` Gate

A release is acceptable when:

1. TypeScript type-checks without errors and the Vite production build completes.
2. Rust builds for the host tests and `wasm32-unknown-unknown` release target.
3. Same normalized parameters replay to the same world/checksum; changed parameters change identity.
4. Every final world parses through `WorldSchema` and all runtime invariants pass.
5. Worker progress, cancellation, failure, and runtime fallback states are visible.
6. Every overlay has units, source-stage provenance, and a semantic legend.
7. Save/archive round trips preserve the complete world and detect tampering.
8. Desktop and narrow layouts keep the map usable without overlapping controls.
9. Documentation describes current backends and scientific limits accurately.

## Runtime Invariants

- Grid: contiguous IDs, finite unit centers, nominal area, unique/symmetric adjacency.
- Plates: complete ownership, connected membership, valid origins and boundary references.
- Boundaries: adjacent cross-plate cells, finite physical motion, normalized potentials.
- Terrain: finite bounded elevation, non-negative erosion/deposition/water, valid decomposition.
- Hydrology: valid neighbor-only downstream pointers, no cycles, valid basins, normalized flow.
- Climate/classification: finite bounded fields, stable biome IDs, valid suitability components/explanations.
- Layers: valid source stages, hashes, min/max, resolution, and backend.
- GPU diagnostic when available: complete finite readback and normalization within tolerance.

## Test Ownership

- `parameters.test.ts`: normalization and draft invalidation.
- `pipeline.test.ts`: full schema, validation, replay, identity, and provenance.
- `topology.test.ts`: grid, plates, and boundary kinematics.
- `hydrology.test.ts`: terrain, drainage, climate, and biome constraints.
- `storage.test.ts`: archive round trip/tamper detection and CSV escaping.
- Rust unit tests: ABI version, randomness, and grid input checks.

## Scientific Honesty

Passing invariants establishes internal coherence, not realism. The app must retain world notes naming grid, geodynamic, hydrologic, climate, resource, and suitability approximations. Claims of realism require external calibration metrics that are not part of `0.2.0`.

## Future Gates

Performance budgets, automated visual regression across viewports, calibrated terrain statistics, cross-runtime WASM parity fixtures for migrated algorithms, and richer archive migrations are release improvements, not silently implied current capabilities.
