use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    artifact,
    cli::DoctorArgs,
    process::{ProcessRequest, capture},
};

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    schema_version: u32,
    captured_unix_ms: u128,
    workspace: PathBuf,
    platform: PlatformReport,
    repository: RepositoryReport,
    tools: Vec<ToolReport>,
}

#[derive(Debug, Serialize)]
struct PlatformReport {
    os: &'static str,
    architecture: &'static str,
    family: &'static str,
    rust_host: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct RepositoryReport {
    revision: Option<String>,
    dirty: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ToolReport {
    name: &'static str,
    capability: &'static str,
    available: bool,
    path: Option<PathBuf>,
    version: Option<String>,
}

struct ToolDefinition {
    name: &'static str,
    executable: &'static str,
    capability: &'static str,
    version_args: &'static [&'static str],
}

const TOOLS: &[ToolDefinition] = &[
    ToolDefinition::new("rustc", "rustc", "Rust compilation", &["-Vv"]),
    ToolDefinition::new("cargo", "cargo", "Cargo orchestration", &["-V"]),
    ToolDefinition::new("rustup", "rustup", "Rust toolchain management", &["-V"]),
    ToolDefinition::new("codex", "codex", "Codex CLI", &["--version"]),
    ToolDefinition::new("lldb", "lldb", "Native debugging", &["--version"]),
    ToolDefinition::new(
        "lldb-dap",
        "lldb-dap",
        "Debugger Adapter Protocol",
        &["--version"],
    ),
    ToolDefinition::new("lldb-mcp", "lldb-mcp", "Debugger MCP", &["--version"]),
    ToolDefinition::new(
        "llvm-symbolizer",
        "llvm-symbolizer",
        "Native stack symbolization",
        &["--version"],
    ),
    ToolDefinition::new("cdb", "cdb", "Windows debugging", &["-version"]),
    ToolDefinition::new("gdb", "gdb", "GNU native debugging", &["--version"]),
    ToolDefinition::new("rr", "rr", "Record and replay", &["--version"]),
    ToolDefinition::new("perf", "perf", "Linux performance counters", &["--version"]),
    ToolDefinition::new(
        "cargo-nextest",
        "cargo-nextest",
        "Isolated test execution",
        &["--version"],
    ),
    ToolDefinition::new(
        "cargo-miri",
        "cargo-miri",
        "Undefined behavior checks",
        &["--version"],
    ),
    ToolDefinition::new(
        "cargo-audit",
        "cargo-audit",
        "Dependency vulnerability checks",
        &["--version"],
    ),
    ToolDefinition::new(
        "cargo-deny",
        "cargo-deny",
        "Dependency policy checks",
        &["--version"],
    ),
    ToolDefinition::new("samply", "samply", "Sampling profiler", &["--version"]),
    ToolDefinition::new(
        "cargo-flamegraph",
        "cargo-flamegraph",
        "Flamegraph capture",
        &["--version"],
    ),
    ToolDefinition::new(
        "tracy-profiler",
        "tracy-profiler",
        "Tracy timeline analysis",
        &["--version"],
    ),
    ToolDefinition::new(
        "renderdoccmd",
        "renderdoccmd",
        "GPU frame capture",
        &["--version"],
    ),
];

impl ToolDefinition {
    const fn new(
        name: &'static str,
        executable: &'static str,
        capability: &'static str,
        version_args: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            executable,
            capability,
            version_args,
        }
    }
}

pub fn run(workspace: &Path, arguments: &DoctorArgs) -> Result<()> {
    let report = collect(workspace);
    let json =
        serde_json::to_string_pretty(&report).context("failed to serialize doctor report")?;
    println!("{json}");

    if let Some(path) = &arguments.output {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            workspace.join(path)
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        artifact::write(&path, json.as_bytes())?;
    }
    Ok(())
}

pub fn collect(workspace: &Path) -> DoctorReport {
    let rust_version = command_text("rustc", &["-Vv"], workspace);
    DoctorReport {
        schema_version: 1,
        captured_unix_ms: artifact::unix_millis(),
        workspace: workspace.to_path_buf(),
        platform: PlatformReport {
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            family: std::env::consts::FAMILY,
            rust_host: rust_version.as_deref().and_then(rust_host),
        },
        repository: repository_report(workspace),
        tools: TOOLS.iter().map(|tool| inspect(tool, workspace)).collect(),
    }
}

pub fn executable_path(name: &str) -> Option<PathBuf> {
    let finder = if cfg!(windows) { "where.exe" } else { "which" };
    if let Ok(output) = std::process::Command::new(finder).arg(name).output()
        && output.status.success()
        && let Some(path) = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
    {
        return Some(PathBuf::from(path));
    }
    known_windows_paths(name)
        .into_iter()
        .find(|path| path.is_file())
}

#[cfg(windows)]
fn known_windows_paths(name: &str) -> Vec<PathBuf> {
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let mut candidates = Vec::new();

    if matches!(name, "lldb" | "lldb-dap" | "lldb-mcp" | "llvm-symbolizer")
        && let Some(root) = program_files
    {
        candidates.push(root.join("LLVM").join("bin").join(format!("{name}.exe")));
    }
    if name == "cdb"
        && let Some(root) = program_files_x86
    {
        candidates.push(
            root.join("Windows Kits")
                .join("10")
                .join("Debuggers")
                .join("x64")
                .join("cdb.exe"),
        );
    }
    if name == "renderdoccmd"
        && let Some(root) = std::env::var_os("ProgramFiles").map(PathBuf::from)
    {
        candidates.push(root.join("RenderDoc").join("renderdoccmd.exe"));
    }
    candidates
}

#[cfg(not(windows))]
fn known_windows_paths(_name: &str) -> Vec<PathBuf> {
    Vec::new()
}

fn inspect(tool: &ToolDefinition, workspace: &Path) -> ToolReport {
    let path = executable_path(tool.executable);
    let version = path
        .as_ref()
        .and_then(|path| command_text(&path.to_string_lossy(), tool.version_args, workspace));
    ToolReport {
        name: tool.name,
        capability: tool.capability,
        available: path.is_some(),
        path,
        version,
    }
}

fn repository_report(workspace: &Path) -> RepositoryReport {
    RepositoryReport {
        revision: command_text("git", &["rev-parse", "--verify", "HEAD"], workspace),
        dirty: command_output("git", &["status", "--porcelain"], workspace)
            .map(|status| !status.trim().is_empty()),
    }
}

fn command_text(program: &str, args: &[&str], workspace: &Path) -> Option<String> {
    command_output(program, args, workspace).filter(|text| !text.is_empty())
}

fn command_output(program: &str, args: &[&str], workspace: &Path) -> Option<String> {
    let request = ProcessRequest {
        program: program.to_owned(),
        args: args.iter().map(ToString::to_string).collect(),
        current_dir: workspace.to_path_buf(),
        env: BTreeMap::default(),
        timeout: Duration::from_secs(5),
    };
    let output = capture(&request).ok()?;
    if output.timed_out || output.exit_code != Some(0) {
        return None;
    }
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    Some(String::from_utf8_lossy(&bytes).trim().to_owned())
}

fn rust_host(version: &str) -> Option<String> {
    version
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::rust_host;

    #[test]
    fn extracts_rust_host_triple() {
        assert_eq!(
            rust_host("rustc 1.95\nbinary: rustc\nhost: x86_64-pc-windows-msvc\n"),
            Some("x86_64-pc-windows-msvc".to_owned())
        );
    }
}
