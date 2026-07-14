# Region And Planet Grid Schema

## Representation

The planet is a dense array of `latBands * lonBands` regions. Latitude centers are uniform in sine latitude, longitude centers are uniform in degrees, and every region receives the same nominal area `4*pi*r^2/count`. This avoids the area weighting error of linearly spaced latitude centers while preserving constant-time array lookup.

Region ID is `latIndex * lonBands + lonIndex`. Longitude wraps. Each region connects to every available cell in its surrounding 3 by 3 index neighborhood, excluding itself; interior degree is eight and polar-row degree is five. There is no cross-pole edge.

## Runtime Ownership

- TypeScript constructs centers, unit vectors, region objects, and adjacency.
- Rust/WASM validates supported dimensions, nominal sphere-area checksum, and wrapped-grid preconditions when available.
- The TypeScript fallback implements the same ABI when WASM cannot load.
- Canvas hit testing converts a map coordinate directly back to the grid index.

## Invariants

- Dimensions are within the parameter schema and multiplication is safe.
- IDs are contiguous and match array positions.
- Every region has at least three unique neighbors.
- Adjacency is symmetric and contains no self edge.
- Longitude seam neighbors are present.
- Sum of nominal areas approximates the sphere area.
- Center vectors are finite and approximately unit length.

## Known Distortion

Equal nominal area does not make this a uniform mesh. Polar cells have different shape and degree from interior cells, and diagonal graph edges have different physical lengths. Simulation code therefore uses great-circle distance wherever a metric distance matters.

The grid feeds [07-tectonics-schema.md](07-tectonics-schema.md), [08-terrain-erosion-hydrology-schema.md](08-terrain-erosion-hydrology-schema.md), and [09-climate-biomes-resources-schema.md](09-climate-biomes-resources-schema.md).
