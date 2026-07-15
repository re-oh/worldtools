# Debugging and Diagnostics

WorldTools debugging is evidence-first. Reproduce with explicit inputs, retain artifacts, narrow the failing boundary, then make one coherent change. This chapter covers the repository's built-in tooling and does not require internet access.

## First response

Use this order unless the symptom clearly demands a narrower path:

1. Run `cargo xtask doctor` and retain the capability report.
2. Select or create a deterministic TOML case under [`.debug/cases`](../../.debug/cases).
3. Reproduce before changing source with `cargo xtask repro <case>` or `cargo xtask capture <case>`.
4. Record no more than three ranked hypotheses. For each, name an observation that would falsify it.
5. Run the cheapest experiment that separates those hypotheses.
6. Capture in-app diagnostics before mutating live state.
7. Patch only the demonstrated cause.
8. Repeat the original case, format, and run `cargo xtask check quick`; broaden checks in proportion to the change.

Do not run concurrent Cargo commands against the same `target` directory.

## The two artifact families

WorldTools keeps application evidence separate from development-harness evidence:

```text
.runtime/diagnostics/       one running app's logs, panic reports, snapshots, audits
.debug/runs/<run-id>/       one bounded xtask repro, capture, or debugger-script run
```

This distinction matters. A diagnostic snapshot describes live ECS/resources at one point in an interactive session. An xtask result describes a controlled external command and whether each attempt met its expected exit behavior.

Both roots are excluded from source-control work. Preserve the relevant directory path in a bug report rather than pasting only a final error line.

## Host capability report

[`cargo xtask doctor`](../../xtask/src/doctor.rs) prints JSON describing the platform, Rust host, repository revision/dirty state, and available development tools. To retain a stable copy:

```powershell
cargo xtask doctor --output .debug/doctor.json
```

The probe includes Git, Rust/Cargo, Codex, LLDB and LLDB integrations, CDB, GDB, rr, perf, nextest, Miri, cargo-audit, cargo-deny, Samply, cargo-flamegraph, Tracy, and RenderDoc. Availability is a capability report, not a requirement that every machine install every tool.

Use the report to choose an investigation lane. For example, do not plan a CDB session until `doctor` confirms CDB; do not claim a GPU capture was taken merely because RenderDoc is named in the workflow.

## Deterministic cases

A case is a strict TOML document parsed by [`xtask/src/case.rs`](../../xtask/src/case.rs). Supported fields are:

```toml
name = "short human-readable label"       # optional
command = ["program", "arg1", "arg2"]    # required
working_directory = "relative/path"       # optional, workspace by default
expected_exit = 0                          # default 0
timeout_seconds = 180                      # default 30, must be nonzero
repeat = 2                                 # default 1, must be nonzero
seed = 91842                               # optional

[env]
RUST_LOG = "worldtools=debug"

[debug]
program = "target/debug/worldtools.exe"
args = []
breakpoints = ["worldtools::main", "src/file.rs:42"]
```

When `seed` is present, xtask supplies `WORLDTOOLS_SEED` unless the case's environment already defines it. It also defaults `RUST_BACKTRACE` to `full`. `repeat` is part of the contract: a flaky or timing-sensitive failure is not resolved by one lucky passing attempt.

The checkout includes:

- [`terrain-smoke.toml`](../../.debug/cases/terrain-smoke.toml): runs `worldtools-lab verify` twice at seed 91842 and includes a native-debug target.
- [`world-history.toml`](../../.debug/cases/world-history.toml): runs simulation library tests twice at the same seed.

Create a new case when the command, seed, timeout, environment, or expected result differs materially. Do not keep changing one case until it no longer represents the original failure.

## Repro versus capture

Both commands use the same bounded runner in [`xtask/src/reproduce.rs`](../../xtask/src/reproduce.rs):

```powershell
cargo xtask repro terrain-smoke
cargo xtask capture terrain-smoke
```

`repro` retains command evidence. `capture` additionally writes the current `doctor.json` into the run directory. A generated directory under `.debug/runs` contains:

| Artifact | Meaning |
|---|---|
| `meta.json` | Case path, command, working directory, expected exit, timeout, repeat, seed, environment key names, start time. |
| `result.json` | Overall success and structured result for each attempt. |
| `attempt-NN-stdout.log` | Exact stdout for one attempt. |
| `attempt-NN-stderr.log` | Exact stderr for one attempt. |
| `stdout.log`, `stderr.log` | Combined logs with attempt separators. |
| `doctor.json` | Host/tool/repository report; present for `capture`. |

The command returns failure if any attempt times out or exits differently from `expected_exit`. `result.json` is updated after each attempt, so partial evidence survives an interrupted multi-attempt run.

## In-app diagnostics

Diagnostics are installed before Bevy's default plugins in [`apps/worldtools/src/main.rs`](../../apps/worldtools/src/main.rs) and implemented by [`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs). The default directory is:

```text
<working-directory>/.runtime/diagnostics
```

Override it with `WORLDTOOLS_LOG_DIR`. Override the application target filter with `WORLDTOOLS_LOG`; the default enables debug events for WorldTools app/render/UI targets and info for world generation. Files are written without ANSI color and include source location, target, thread, and span-close events.

Example PowerShell launch with a dedicated evidence directory:

```powershell
$env:WORLDTOOLS_LOG_DIR = ".debug/runs/manual-map-session"
$env:WORLDTOOLS_LOG = "worldtools=debug,worldtools_render=trace"
cargo run -p worldtools
```

The directory can contain:

- `worldtools.log.YYYY-MM-DD`: rolling daily trace log.
- `panic-<timestamp>-<pid>.txt`: build/platform/process metadata, panic location and message, diagnostic environment, and forced backtrace.
- `snapshot-<timestamp>.json`: live application, system, document, simulation, view, tile, renderer, layer, and recent-event state.
- `terrain-audit-<timestamp>.json`: deterministic terrain distribution, seam, parent-child LOD, finite-value, and repeatability results.

Snapshot and audit JSON is written through a temporary file and atomically renamed, so a final filename represents a complete serialization.

## Diagnostics window

Press F12 to open the native diagnostics window implemented in [`debug_window.rs`](../../crates/worldtools_ui/src/shell/debug_window.rs). It has five tabs:

- **Summary**: FPS, frame time, frame number, entities, process CPU/RAM, rendered/degraded page counts, generation activity, snapshot, and terrain-audit actions.
- **Streaming**: LOD, visible/resident/in-flight/ready counts, request/completion/discard/invalidation counters, generation timings, cache/queue limits, render states, overlays, freeze, and cache flush.
- **Viewport**: geographic center, vertical span, logical/physical dimensions, pixels per point, LOD, and ground resolution.
- **Layers**: native layer availability and current selection.
- **Events**: bounded, filterable in-memory trace events with dropped-event count.

The event path is deliberately non-blocking. The tracing layer has a 2,048-event channel; the app drains at most 256 per frame into a 512-entry UI log. Drops are counted at both boundaries. A high drop count means the visible event list is incomplete; use the rolling file log as the retained source.

## Streaming diagnostics

The most useful renderer controls map directly to [`RenderDebugSettings`](../../crates/worldtools_render/src/debug.rs):

- **Tile borders** shows exact projected page edges.
- **LOD tint** colors desired LODs with a stable palette.
- **Residency tint** distinguishes exact, ancestor fallback, and stale pages.
- **Trace lifecycle** emits tile planning, request, generation, acceptance/discard, and surface-state events.
- **Freeze tile streaming** stops new requests while allowing in-flight work to complete.
- **Flush tile cache** invalidates resident and in-flight IDs so revisions reject late results and visible pages are requested again.

Interpret counters together:

- `visible` is visual placements, including multiple wrapped placements of one canonical page.
- `resident_visible` includes exact pages and usable ancestors.
- `resident_total` is the CPU cache entry count; capacity is 128.
- `in_flight` is active jobs; scheduler limit is 8.
- `ready_results` is the completed-result channel depth; the app consumes at most 8 per frame.
- `discarded` should rise when layer changes or invalidations make late work stale. A rising value alone is not a correctness failure.
- `fallback`, `stale`, and `missing` describe what was submitted to the GPU, not merely CPU cache contents.

Capture a snapshot before toggling freeze or flushing the cache if the current state is important.

## Diagnostic snapshots and terrain audits

**Snapshot** emits schema `worldtools.diagnostic-snapshot.v2` from [`snapshot.rs`](../../apps/worldtools/src/debug_tools/snapshot.rs). It includes the world fingerprint and simulation settings, view, UI state, telemetry, stream/render stats, sorted resident and in-flight IDs with revisions, layer capabilities, system info, and up to 200 recent events.

Use the fingerprint to compare generated-world identity across sessions. Use `world_epoch`, tile revision, source layer, and recent events to distinguish an old result from the current world.

**Terrain audit** runs asynchronously from [`audit.rs`](../../apps/worldtools/src/debug_tools/audit.rs). It checks all six root cube faces for finite terrain distribution, a known cross-face seam, a parent-child LOD relationship, and deterministic regeneration. Its strict pass condition requires zero-bit error for the tested seam and LOD pair and no non-finite elevation samples.

The audit diagnoses world-generation consistency. It does not prove that viewport placement, GPU upload, or shader presentation is correct; use render overlays and screenshots for those boundaries.

## Live Bevy Remote inspection

The optional endpoint is implemented in [`live_remote.rs`](../../apps/worldtools/src/live_remote.rs) and compiled only with the `live-debug` feature:

```powershell
$env:WORLDTOOLS_BRP = "1"
cargo run -p worldtools --features live-debug
```

It binds to IPv4 loopback, is disabled unless explicitly enabled, and refuses to start in a release build. The custom read-only `worldtools.status` method returns document, view, simulation, streaming, renderer, performance, and debug state.

Mutation methods are rejected by default. Writable access requires a second explicit opt-in before launch:

```powershell
$env:WORLDTOOLS_BRP_ALLOW_WRITE = "1"
```

Start with status and read-only ECS queries. Take a diagnostic snapshot first. Resource/component mutation, events, spawning, despawning, and reparenting change the evidence and should be used only with explicit intent.

## Native debugger scripts

Generate a noninteractive LLDB or CDB command file from a case:

```powershell
cargo xtask debug-script terrain-smoke --backend lldb
cargo xtask debug-script terrain-smoke --backend cdb
cargo xtask debug-script terrain-smoke --backend cdb --output .debug/session.cdb
```

Add `--run` to start the selected debugger after generation. The implementation is in [`debug_script.rs`](../../xtask/src/debug_script.rs). LLDB scripts configure the working directory, target arguments, environment, source/symbol breakpoints, then capture all thread backtraces and frame variables. CDB scripts break on access violations, apply symbol breakpoints, then capture all stacks and locals.

Attach a native debugger only after narrowing the smallest useful breakpoint or watchpoint. At a stop, inspect the triggering thread, full stack, arguments, relevant locals, and the violated invariant. For high-frequency systems, prefer conditional breakpoints or tracepoints.

## Symptom-to-tool guide

| Symptom | First evidence | Best next tool | What would discriminate the cause |
|---|---|---|---|
| App exits or panics | Panic report and rolling log | `cargo xtask capture <case>`; LLDB/CDB script after narrowing | Repeated stack/location with the same seed versus host-specific failure. |
| Generated world differs between runs | Snapshot fingerprints and settings | `world-history` case; terrain audit | Same inputs with different fingerprints or audit determinism false. |
| Crack at a tile boundary | Screenshot plus tile borders | Residency/LOD tints; terrain audit; shader/source inspection | CPU seam audit zero but visible crack implicates sampling, apron, transform, or GPU presentation. |
| Flashing tiles after layer/world change | Snapshot before change, trace lifecycle | Freeze streaming; inspect revisions, layer, `world_epoch`, discarded count | Accepted result with mismatched epoch/revision/layer versus correctly discarded late work. |
| Blank holes while moving | Streaming and renderer telemetry | Residency tint; trace lifecycle; snapshot | `missing > 0` with no roots/in-flight points to scheduling; resident pages with holes points to surface/GPU state. |
| Permanently blurry region | LOD tint and exact/fallback counts | Snapshot tile IDs; flush cache once | Desired LOD never requested/accepted versus exact page resident but sampled incorrectly. |
| Map offset or wrong size | Viewport tab logical/physical sizes | Inspect UI `MapViewport`, bridge, render `MapViewport` | Difference caused by egui zoom/window scale conversion rather than projection. |
| Map responds through UI | `input_blocked`, hovered state | Inspect egui viewport ownership and navigation settings | Map starts a gesture while egui reports pointer ownership. |
| Wrong colors for a layer | Active layer/mode and snapshot layer state | Inspect bridge, channel contract, shader mode | Correct layer page with wrong shader interpretation versus stale previous-layer page. |
| Frame-time spike during movement | Frame telemetry and generation timings | Optimized stable workload with Samply/ETW/perf/Tracy | CPU tile generation, result upload, ECS/surface churn, or GPU work correlates with spike. |
| GPU-only corruption or validation error | Screenshot, wgpu log, tile state snapshot | RenderDoc capture | Correct ECS/material handles but incorrect texture/bind-group/draw state in captured frame. |
| Test/compiler failure | Full stderr and exact command | Focused test or nextest after `doctor` | Minimal package/filter reproduces independently of unrelated targets. |
| Suspected undefined behavior | Small deterministic case | Miri or applicable sanitizer | Failure under instrumentation at the same operation. |

For a visual defect, correlate at least four layers of evidence: viewport/camera state, visible/resident tile state, screenshot, and wgpu/RenderDoc evidence. A screenshot alone cannot distinguish bad source data from bad placement or shader interpretation.

For a performance defect, reproduce in an optimized build with a fixed seed and stable interaction. Source shape is not profiling evidence.

## Verification commands

The sequential verification harness is implemented in [`xtask/src/check.rs`](../../xtask/src/check.rs):

```powershell
cargo fmt --all
cargo xtask check quick
cargo xtask check full
cargo xtask check full --miri
```

`quick` checks formatting, the whole workspace/all targets, Clippy with warnings denied, and workspace library tests. `full` replaces library-only tests with all workspace targets. Miri is accepted only with `full` and requires the appropriate nightly/tool component.

For a narrow Rust change, run a focused package test first when available:

```powershell
cargo nextest run -p <package> <filter>
```

Then run the original reproduction at least as many times as its case specifies. In the final report, name the failing-before command, passing-after command, evidence directory, demonstrated root cause, and remaining uncertainty.

## Source map

- App assembly and diagnostics: [`main.rs`](../../apps/worldtools/src/main.rs), [`diagnostics.rs`](../../apps/worldtools/src/diagnostics.rs), [`debug_tools`](../../apps/worldtools/src/debug_tools/mod.rs)
- Renderer telemetry and controls: [`streaming.rs`](../../crates/worldtools_render/src/streaming.rs), [`debug.rs`](../../crates/worldtools_render/src/debug.rs), [`tile_surface.rs`](../../crates/worldtools_render/src/tile_surface.rs)
- UI diagnostics model and window: [`model/debug.rs`](../../crates/worldtools_ui/src/model/debug.rs), [`shell/debug_window.rs`](../../crates/worldtools_ui/src/shell/debug_window.rs)
- Deterministic harness: [`xtask/src/cli.rs`](../../xtask/src/cli.rs), [`case.rs`](../../xtask/src/case.rs), [`reproduce.rs`](../../xtask/src/reproduce.rs), [`artifact.rs`](../../xtask/src/artifact.rs)
- Rendering architecture: [Rendering, UI, and Runtime Integration](rendering-ui.md)
