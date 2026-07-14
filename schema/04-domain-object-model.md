# Domain Object Model

The canonical model is implemented with Zod in `src/schema/domain.ts`. UI panels and exports consume these objects or typed projections; they do not create parallel data shapes.

## Version Identity

Both `SCHEMA_VERSION` and `APP_VERSION` are `0.2.0`. A deterministic world identity includes its normalized parameters. Timestamps and measured performance do not participate in replay identity.

## Project And World

`Project` indexes locally saved world IDs and the active ID. `World` owns identity, name, seed, versions, timestamps, normalized parameters, `Planet`, regions, plates, contacts, pipeline statuses, simulation-layer descriptors, validation results, performance measurements, and scientific limitation notes.

## Planet

`Planet` stores radius in kilometers, sea level in meters, axial tilt in degrees, day length in hours, projection mode, and grid dimensions/checksum. The current projection literal is `equal-area-spherical-grid-v1`; the legacy equirectangular literal remains readable for migration.

## Region

Every region stores:

| Group | Fields |
| --- | --- |
| Geometry | `regionId`, center latitude/longitude, unit `centerVector3`, nominal `areaKm2`, symmetric `neighborIds` |
| Tectonics | `plateId`, `crustType`, base and tectonic elevation in meters |
| Surface | surface elevation, normalized slope, erosion/deposition/sediment depth, water depth |
| Drainage | runoff in mm/year, discharge in km3/year, normalized flow accumulation, downstream region, basin ID |
| Climate | ocean distance in km, mean/range temperature in C, precipitation in mm/year, aridity, wind direction, climate class |
| Classification | biome ID/name, resource IDs, one explained `SuitabilityScore` |

All numeric fields must be finite. Depths and discharge are non-negative, normalized values stay in `0..1`, and every referenced region ID must exist.

## Plate And BoundarySegment

`Plate` stores an origin region, crust bias, unit Euler-pole vector, angular speed in degrees per million years, age in Ma, owned region IDs, and boundary IDs.

`BoundarySegment` represents one cross-plate neighbor pair. It stores plate/region IDs, type, signed convergence and non-negative shear in centimeters per year, plus normalized uplift, trench, volcanism, and earthquake potentials. Runtime preprocessing maps legacy motion and velocity fields into this representation.

## SimulationLayer

A layer descriptor records ID/type, schema version, region resolution, storage kind, source stage, units, data reference, min/max, parameter hash, input hashes, generation time, and truthful runtime backend. The numeric arrays remain canonical region fields in `0.2.0`; layer descriptors provide provenance and discovery rather than duplicating data.

## Pipeline And Validation

Stage statuses support `fresh`, `stale`, `running`, `failed`, `skipped`, and `cancelled`, with progress, duration, backend, message, and timestamp. Validation results are named booleans with human-readable detail.

See [05-simulation-pipeline.md](05-simulation-pipeline.md) for production order and [14-data-contracts-and-storage.md](14-data-contracts-and-storage.md) for persisted envelopes.
