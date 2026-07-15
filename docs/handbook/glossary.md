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

