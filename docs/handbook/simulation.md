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

