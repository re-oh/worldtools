# Schema-Driven Development Workflow

## Principle

Schemas are the source of truth. Code implements schemas; code does not quietly invent domain fields, buffer layouts, lifecycle states, or UI meanings.

## Required Schema Layers

1. Domain schema: named entities, fields, units, invariants.
2. Pipeline schema: stage inputs, outputs, dependencies, and invalidation rules.
3. Runtime schema: TypeScript state, Rust/WASM interfaces, WebGPU buffers.
4. UI schema: panels, controls, overlays, selection behavior, and inspection views.
5. Validation schema: tests, metrics, debug views, and acceptance gates.

## Development Loop

1. Write or update markdown schema.
2. Add TypeScript type or Zod schema.
3. Add Rust struct or WASM interface when CPU-side.
4. Add WGSL buffer layout when GPU-side.
5. Add fixture data.
6. Add validation checks.
7. Implement feature.
8. Add UI inspection for generated data.
9. Mark done only when the schema, implementation, and validation agree.

## Naming Rules

- Use singular names for entity schemas: `Region`, `Plate`, `BoundarySegment`.
- Use plural names for collections: `RegionStore`, `PlateSet`.
- Include units in field names or metadata: `elevationMeters`, `velocityCmPerYear`, `temperatureC`.
- Avoid overloaded terms. If a term differs from geology usage, document the internal meaning.
- Use stable IDs for all persistent entities.

## Schema File Template

Every implementation schema should answer:

- Purpose: what decision or workflow this schema supports.
- Owner: TypeScript, WGSL, Rust/WASM, storage, or UI.
- Fields: names, types, units, constraints.
- Invariants: what must always be true.
- Dependencies: upstream schemas or simulation stages.
- Derived outputs: downstream data generated from it.
- Done checks: tests, visual checks, or metrics.

## Change Control

A schema change is breaking when it changes persisted fields, GPU buffer layout, Rust/WASM ABI, replay determinism, or user-facing interpretation. Breaking changes require:

- Version bump.
- Migration path.
- Fixture update.
- Release note in the schema folder.

## Recursive Review Rule

After creating or changing any schema file, review it against every upstream and downstream file it references:

1. Check whether each concept has a home document.
2. Check whether each home document links back to its consumers.
3. Check whether each runtime boundary is represented in TypeScript, WebGPU, Rust/WASM, UI, storage, and validation when relevant.
4. Add or update cross-links when a concept spans multiple files.
5. Record unresolved design tension as a validation or research gate instead of leaving it implicit.

Use [17-recursive-document-review.md](17-recursive-document-review.md) as the current audit pattern.

## Finish Definition

The workflow is done when a new contributor can add a feature by editing schemas first, then implementing generated or manually mirrored TypeScript, Rust, WGSL, UI, and tests.
