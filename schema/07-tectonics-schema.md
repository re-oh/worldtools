# Tectonics Schema

Tectonics is a deterministic regional plausibility model implemented in `src/simulation/tectonics.ts`. It creates causally related crust, contacts, relief, and hazards without claiming a reconstruction of real geologic time.

## Plate Seeding And Growth

The first origin is seeded directly. Each later origin chooses the most separated of 48 deterministic candidates. Plates receive a shuffled continental, mixed, or oceanic crust bias.

A shared min-heap grows plate frontiers. Step cost combines angular distance from origin, directional anisotropy, deterministic local variation, and a small continental latitude bias. A cell is claimed only from a claimed neighbor, which guarantees connected ownership. Crust type then combines plate bias with spherical fractal noise.

## Motion

Each `Plate` stores a unit `eulerPoleVector3` and `angularVelocityDegreesPerMa` in `0.12..0.90`. Surface velocity at a boundary midpoint is the cross product of Euler pole and midpoint, scaled by angular speed and planet radius into centimeters per year.

## Boundary Segments

Each neighboring cross-plate pair produces one segment. Relative tangent velocity is decomposed into signed `convergenceCmPerYear` and non-negative `shearCmPerYear`. Thresholds classify:

- convergent when positive cross-contact motion dominates;
- divergent when negative cross-contact motion dominates;
- transform when shear dominates;
- oblique otherwise.

Uplift depends on convergence and crust pairing. Trench potential requires oceanic crust. Volcanism combines subduction, divergence, and a small shear term; earthquake potential combines convergence and shear. All potentials are clamped to `0..1`.

## Terrain Coupling

Maximum uplift influence diffuses four graph rings, trench/volcanism about three, and earthquake potential two, each with a fixed decay. This produces broader causal belts while preserving traceability to source contacts.

## Invariants And UI

- Every region belongs to exactly one existing plate.
- Every plate is graph-connected and contains its origin.
- Contact region IDs are adjacent, ordered once, and owned by distinct plates.
- Motion values are finite and use explicit physical units.
- Plate and boundary overlays expose classification and potentials without calling the model a fault reconstruction.

The generated signals feed [08-terrain-erosion-hydrology-schema.md](08-terrain-erosion-hydrology-schema.md) and resource/hazard scoring in [09-climate-biomes-resources-schema.md](09-climate-biomes-resources-schema.md).
