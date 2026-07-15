# WorldTools

WorldTools is a native Rust and Bevy high-fidelity procedural world generator.

The source-level [offline handbook](docs/handbook/README.md) is designed to be read without an internet connection.

## Workspace

- `apps/worldtools`: native desktop application
- `crates/worldtools_world`: deterministic world data and terrain generation
- `crates/worldtools_render`: WebGPU renderer, LOD tiles, and streaming
- `crates/worldtools_ui`: compact egui editor shell
- `crates/worldtools_analysis`: generation quality analysis
- `tools/worldtools_lab`: offline terrain analysis
- `xtask`: reproducible debugging and validation harness

## Development

```powershell
cargo run -p worldtools
cargo xtask doctor
cargo xtask check quick
```

Press `F12` in the application to open diagnostics. Development-only Bevy Remote is enabled with the `live-debug` feature and `WORLDTOOLS_BRP=1`.
