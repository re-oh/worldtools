# WorldTools Agent Guide

## Engineering constraints

- Keep modules focused. Split code when ownership, lifecycle, or testing boundaries differ.
- Prefer existing workspace crates and proven ecosystem crates over local substitutes.
- Keep the simulation deterministic: every generated result must be reproducible from explicit inputs and seed data.
- Treat Windows MSVC and native Linux as supported execution lanes. Do not hide platform-specific failures.
- Never run concurrent Cargo builds against the same `target` directory.
- Do not weaken tests, validation, logging, or assertions to make a failure disappear.

## Debugging protocol

1. Run `cargo xtask doctor` and retain its capability report.
2. Create or select a deterministic case under `.debug/cases/`.
3. Reproduce the failure before changing source.
4. Capture stdout, stderr, metadata, traces, and relevant snapshots with `cargo xtask capture <case>`.
5. Maintain at most three ranked hypotheses. For each, name the observation that would falsify it.
6. Run the cheapest discriminating experiment first.
7. Use WorldTools diagnostics and Bevy Remote to narrow the responsible state or system.
8. Use LLDB only after identifying the smallest useful breakpoint or watchpoint.
9. Make one coherent source change that addresses the demonstrated cause.
10. Repeat the original reproduction, then run `cargo xtask check quick` and broader checks proportional to risk.

Report the failing-before command, passing-after command, evidence directory, and remaining uncertainty.

## Agent roles

Use subagents for bounded, read-only exploration, evidence analysis, or independent verification when parallel work is useful. The root agent owns source edits unless it explicitly delegates a disjoint module. Do not have multiple agents speculate on fixes in the same files.

## Live debugging

- Keep Bevy Remote bound to loopback and disabled unless explicitly requested.
- Start with read-only ECS queries. Mutation, event triggering, spawning, and despawning require explicit user intent.
- Prefer fixed seeds, paused simulation, and single-frame stepping for state-transition bugs.
- Capture a diagnostics snapshot before changing live state.
- Use conditional breakpoints or tracepoints for high-frequency systems.
- For visual defects, correlate ECS state, WorldTools snapshots, screenshots, wgpu validation output, and RenderDoc captures.
- For performance defects, reproduce in an optimized build and use Tracy, Samply, `perf`, or ETW before changing algorithms.

## Validation

- Fast lane: `cargo xtask check quick`
- Full lane: `cargo xtask check full`
- Focused Rust tests: `cargo nextest run -p <package> <filter>` when nextest is installed
Run formatting before final verification. Keep generated runs, logs, snapshots, and captures out of source control.
