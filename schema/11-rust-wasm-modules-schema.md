# Rust/WASM Module Schema

Rust/WASM is deliberately narrow in `0.2.0`. It supplies deterministic primitives and grid preflight checks; the TypeScript worker owns graph construction and all simulation stages.

## ABI

`crates/bombo_core` exports numeric functions only:

| Export | Contract |
| --- | --- |
| `bombo_schema_version()` | returns integer schema generation `200` |
| `bombo_seed_hash(seed, version)` | deterministic 32-bit seed expansion |
| `bombo_random_at(seed, stream, index, salt)` | deterministic indexed random word |
| `bombo_grid_region_count(lat, lon)` | validated cell count or zero |
| `bombo_grid_area_checksum(lat, lon, radius)` | rounded nominal sphere area checksum or zero |
| `bombo_validate_wrapped_grid(lat, lon)` | zero on accepted dimensions, non-zero error code otherwise |

Grid bounds are `3..256` latitude bands and `4..512` longitude bands. Invalid radius, non-finite radius, and multiplication failures return an error value rather than trapping.

## Loading And Fallback

`src/wasm/loader.ts` fetches `/wasm/bombo_core.wasm`, attempts streaming instantiation, and retries from bytes for servers with an incorrect MIME type. Every expected export is checked before the module is accepted.

A TypeScript compatibility object implements the same interface and hash function. Failure to fetch or instantiate WASM is visible in runtime status, but deterministic generation continues. Stage provenance therefore reports the seed/grid preflight as mixed rather than claiming that Rust owns plate or hydrology work.

## Build And Tests

`npm run build:wasm` compiles the release artifact for `wasm32-unknown-unknown` and copies it to `public/wasm`. Rust unit tests cover schema generation, indexed randomness, and grid rejection. TypeScript integration tests exercise the fallback through complete world generation.

## Expansion Rule

Moving a graph algorithm into Rust is optional future work. It requires a flat typed-array ABI, matching TypeScript behavior, deterministic cross-runtime fixtures, explicit error codes, measured benefit, and truthful stage provenance before the existing CPU implementation can be replaced.
