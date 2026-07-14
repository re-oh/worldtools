# Data Contracts And Storage

## Active And Local State

The active world is held in memory. Complete validated worlds are stored in IndexedDB through `idb-keyval` under versioned project and world keys. The project record stores world IDs and active identity; list summaries are derived from validated worlds rather than persisted as a competing cache.

Invalid saved records are skipped in the library but left untouched for future recovery tooling. Save updates the stored world's timestamp without mutating the active in-memory instance.

## Native Archive

Archive format `project-bombo-world`, version `2`, is JSON with:

- format/version and export timestamp;
- deterministic world checksum;
- manifest containing identity, names, versions, parameters, stage statuses, and layer references;
- complete schema-validated world.

For current-schema archives, checksum mismatch is a hard error. Recognized legacy envelopes are normalized, and a differing world schema triggers deterministic regeneration from preserved parameters instead of pretending old derived fields are current.

## Exports

- `.bombo.json`: reloadable native archive.
- `-manifest.json`: provenance and parameter metadata without the full region graph.
- `-regions.csv`: escaped tabular region fields with explicit headers.
- `.png`: current Canvas map snapshot.

Blob URLs are short-lived and revoked after download. CSV fields use standard quote doubling. JSON is UTF-8 and human-readable.

## Layer Provenance

Each `SimulationLayer` records source stage, schema version, region count, units, min/max, parameter hash, input hashes, generation timestamp, data reference, and runtime backend. In `0.2.0` the values reside on region objects and layers use `memory` storage references; no binary layer compression is claimed.

## Migration Rules

- External data always passes runtime validation.
- Migration preserves seed, name where possible, and normalized parameter intent.
- Original IndexedDB records are not deleted on parse failure.
- App/schema versions and archive format versions are independent explicit fields.
