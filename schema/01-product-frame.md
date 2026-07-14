# Product Frame

## Product Name

Project Bombo

## Product Type

A scientific-fantasy world simulation workbench. It lets users generate, inspect, compare, and tune plausible Earth-like worlds from first principles: plate tectonics, terrain deformation, erosion, hydrology, climate, biomes, resources, and settlement suitability.

## Design Intent

The app is not a map painter. It is a schema-driven simulator with a readable chain of causality:

1. Planet grid defines regions.
2. Plate growth and plate motion define crust and boundaries.
3. Tectonic interactions produce elevation, basins, arcs, trenches, rifts, and mountain belts.
4. Erosion and hydrology reshape terrain.
5. Atmospheric and latitude rules produce temperature, moisture, and circulation bands.
6. Climate and land qualities produce biomes, soils, resources, and suitability.
7. The UI explains why the generated world looks the way it does.

## Primary Users

- Worldbuilders who want plausible continents, mountains, rivers, biomes, and settlement zones.
- Designers building strategy, RPG, simulation, or survival worlds.
- Technical artists and procedural-generation engineers.
- Curious users who want to inspect the causal model, not only export an image.

## Non-Goals

- Full geodynamic simulation of Earth.
- Real-time physically complete climate modeling.
- An opaque generator with only seed and style sliders.
- A GIS replacement.
- A game engine.

## Product Promise

Every visual feature should have an inspectable reason. A mountain range, desert belt, volcanic island chain, river basin, biome, ore zone, or settlement region should trace back to typed simulation data.

## Schema Connections

- Product scope is grounded in [02-source-concepts.md](02-source-concepts.md).
- The causal chain becomes executable in [05-simulation-pipeline.md](05-simulation-pipeline.md).
- The nouns in the promise are formalized in [04-domain-object-model.md](04-domain-object-model.md).
- The user-facing explanation requirement is carried into [13-ui-ux-schema.md](13-ui-ux-schema.md).
- The release-level definition of done is in [15-quality-validation-and-done.md](15-quality-validation-and-done.md).

## Finish Definition For Product Frame

This frame is done when every implementation issue can be classified as one of:

- A domain schema issue.
- A simulation pipeline issue.
- A rendering issue.
- A UI/UX issue.
- A validation issue.
- Out of scope.
