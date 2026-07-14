# UI And UX Schema

The application is a dense operational workbench. Its first screen is the usable map, not a marketing surface. The interface exposes causal provenance and scientific limits without requiring the user to read implementation documentation.

## Layout

- Header: identity, save/library/export commands, runtime status, responsive drawer controls.
- Parameter rail: resolution presets, seed, plates, erosion, sea level, and suitability use.
- Map workspace: overlay strip, interactive canvas, generation state, map tools, legend, and summary.
- Inspector rail: region, plate, comparison, and validation tabs.
- Pipeline rail: ordered stage status and details.
- Library dialog: local saves, import, load, and delete.

Desktop uses stable side rails around the map. Narrow screens convert rails to drawers while keeping the map primary. Controls use icons, segmented modes, numeric inputs/sliders, selects, progress, and native dialog semantics.

## Interaction

- Hover previews a region; click selects it; Shift-click pins a comparison region.
- Drag pans, wheel zooms, keyboard arrows pan, and plus/minus zoom.
- Overlay changes preserve selection and view state.
- Parameter edits are drafts. Their first affected pipeline stage becomes stale until generation.
- Inputs are disabled during generation, and the worker can be cancelled.
- Loading another world is blocked while generation is active to prevent state races.
- Errors, fallback runtime state, and archive verification are visible notices/status.

## Inspection

Region inspection includes coordinates, plate/crust, terrain decomposition, drainage, climate, biome, resources, and explained suitability. Plate inspection summarizes motion, crust mix, contacts, and hazards. Comparison contrasts two regions in the same world. Validation shows every named invariant result.

Every overlay includes semantic colors, units, source stage, and provenance. Text remains compact within panels; the UI avoids decorative nested cards, gradients, and instructions embedded as feature marketing.

## Accessibility And Resilience

Buttons have labels/tooltips, tabs expose selected state, the canvas is focusable and labeled, generation uses native progress, and notices use a live status region. Fatal startup errors are inserted as text rather than interpolated HTML.
