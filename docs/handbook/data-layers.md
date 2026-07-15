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

