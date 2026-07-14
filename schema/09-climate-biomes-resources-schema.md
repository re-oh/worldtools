# Climate, Biomes, Resources, And Suitability Schema

These stages translate terrain and drainage into inspectable environmental and use-specific signals. They are deterministic CPU models in `src/simulation/climate.ts` and `classification.ts`.

## Climate

Graph Dijkstra distance from all ocean cells yields `distanceToOceanKm`. Temperature combines latitude, a `6.2 C/km` lapse rate, coastal moderation, and low-amplitude spherical noise. Annual temperature range rises with continentality and latitude.

Rainfall combines equatorial and mid-latitude wet belts, subtropical and polar drying, exponential ocean moisture supply, prevailing-wind direction by latitude band, one-cell orographic lift/shadow, and deterministic variation. Aridity compares precipitation with a temperature-derived evapotranspiration proxy.

Stored climate fields are mean and range temperature in C, precipitation and runoff in mm/year, ocean distance in km, wind direction in degrees, aridity in `0..1`, and a named climate class. Climate rainfall recalculates discharge through the existing drainage graph.

## Biomes And Resources

Ten stable biome IDs are selected from water state, temperature, aridity, and precipitation. Resources are potential signals, not reserves. Current IDs are derived from local volcanic/convergent setting, coast position, discharge, biome, and elevation and include metallic ore, geothermal, volcanic soil, sedimentary basin, freshwater, timber, and marine access potentials.

## Suitability

One explicit `useCaseId` is scored at a time:

- rain-fed agriculture;
- river settlement;
- pastoral range;
- mining camp;
- port city;
- defensive hill.

Each use case supplies weights for physical comfort, water, soil, resources, and access. Tectonic/volcanic/slope hazard is subtracted, contextual modifiers are added, and the result is clamped to `0..100`. Stored explanation references name the dominant physical, water, setting, resource, hazard, and biome inputs.

## Invariants And Limits

Climate fields are finite and bounded, biome IDs/names are stable, resource IDs are non-empty strings, and suitability components remain in `0..100`. The model is comparative and causal but does not simulate ocean currents, seasonal atmosphere, soil chemistry, markets, infrastructure, or policy.
