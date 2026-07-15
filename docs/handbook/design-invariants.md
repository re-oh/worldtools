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

- Do not make a stage read “channel Z.”
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

