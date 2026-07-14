use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "WorldTools development and debugging harness")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report the host, toolchain, and debugger capabilities as JSON.
    Doctor(DoctorArgs),
    /// Run a deterministic reproduction case and preserve its result.
    Repro(CaseArgs),
    /// Run a case and collect additional environment evidence.
    Capture(CaseArgs),
    /// Run the repository verification pipeline sequentially.
    Check(CheckArgs),
    /// Generate a noninteractive debugger command script for a case.
    DebugScript(DebugScriptArgs),
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Also write the JSON report to this path.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CaseArgs {
    /// Case name from .debug/cases, or a direct TOML path.
    pub case: PathBuf,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CheckProfile {
    Quick,
    Full,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(value_enum)]
    pub profile: CheckProfile,
    /// Include the targeted Miri library-test lane (nightly only).
    #[arg(long)]
    pub miri: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DebugBackend {
    Lldb,
    Cdb,
}

#[derive(Debug, Args)]
pub struct DebugScriptArgs {
    /// Case name from .debug/cases, or a direct TOML path.
    pub case: PathBuf,
    #[arg(long, value_enum, default_value_t = DebugBackend::Lldb)]
    pub backend: DebugBackend,
    /// Script destination. Defaults to the current debug run directory.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Start the debugger after generating the script.
    #[arg(long)]
    pub run: bool,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{CheckProfile, Cli, Command, DebugBackend};

    #[test]
    fn parses_debug_script_backend() {
        let cli =
            Cli::try_parse_from(["xtask", "debug-script", "terrain-smoke", "--backend", "cdb"])
                .expect("valid CLI");

        let Command::DebugScript(arguments) = cli.command else {
            panic!("wrong command");
        };
        assert!(matches!(arguments.backend, DebugBackend::Cdb));
    }

    #[test]
    fn parses_full_check() {
        let cli = Cli::try_parse_from(["xtask", "check", "full", "--miri"]).expect("valid CLI");

        let Command::Check(arguments) = cli.command else {
            panic!("wrong command");
        };
        assert!(matches!(arguments.profile, CheckProfile::Full));
        assert!(arguments.miri);
    }
}
