use std::path::Path;

use anyhow::{Result, bail};

use crate::{
    cli::{CheckArgs, CheckProfile},
    process::run_inherited,
};

pub fn run(workspace: &Path, arguments: &CheckArgs) -> Result<()> {
    if arguments.miri && !matches!(arguments.profile, CheckProfile::Full) {
        bail!("the Miri lane is only available with `check full`");
    }

    run_step(workspace, "format", &["fmt", "--all", "--", "--check"])?;
    run_step(
        workspace,
        "type-check",
        &["check", "--workspace", "--all-targets"],
    )?;
    run_step(
        workspace,
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;

    match arguments.profile {
        CheckProfile::Quick => run_step(
            workspace,
            "library tests",
            &["test", "--workspace", "--lib"],
        )?,
        CheckProfile::Full => run_step(
            workspace,
            "all workspace tests",
            &["test", "--workspace", "--all-targets"],
        )?,
    }

    if arguments.miri {
        run_step(
            workspace,
            "Miri library tests",
            &["miri", "test", "--workspace", "--lib"],
        )?;
    }
    Ok(())
}

fn run_step(workspace: &Path, label: &str, args: &[&str]) -> Result<()> {
    println!("\n==> {label}");
    run_inherited("cargo", args, workspace)
}
