use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{
    artifact,
    case::{DebugCase, DebugTarget},
    cli::{DebugBackend, DebugScriptArgs},
    doctor,
    process::{run_inherited, run_inherited_with_env},
    workspace,
};

pub fn run(workspace_root: &Path, arguments: &DebugScriptArgs) -> Result<()> {
    let case_path = workspace::resolve_case(workspace_root, &arguments.case);
    let case = DebugCase::load(&case_path)?;
    let target = target_for(&case);
    let working_directory = case.working_directory(workspace_root);
    let extension = match arguments.backend {
        DebugBackend::Lldb => "lldb",
        DebugBackend::Cdb => "cdb",
    };
    let output = match &arguments.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => workspace_root.join(path),
        None => artifact::create_run_directory(workspace_root, "debug-script")?
            .join(format!("session.{extension}")),
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let script = match arguments.backend {
        DebugBackend::Lldb => lldb_script(&case, &target, &working_directory),
        DebugBackend::Cdb => cdb_script(&target),
    };
    artifact::write(&output, script.as_bytes())?;
    println!("debug script: {}", output.display());

    if arguments.run {
        execute(
            arguments.backend,
            &output,
            &case,
            &target,
            &working_directory,
        )?;
    }
    Ok(())
}

fn target_for(case: &DebugCase) -> DebugTarget {
    case.debug.clone().unwrap_or_else(|| DebugTarget {
        program: case.command[0].clone().into(),
        args: case.command[1..].to_vec(),
        breakpoints: Vec::new(),
    })
}

fn lldb_script(case: &DebugCase, target: &DebugTarget, working_directory: &Path) -> String {
    let mut lines = vec![
        "settings set auto-confirm true".to_owned(),
        format!(
            "platform settings -w {}",
            lldb_quote(&working_directory.to_string_lossy())
        ),
        format!(
            "target create {}",
            lldb_quote(&target.program.to_string_lossy())
        ),
    ];
    if !target.args.is_empty() {
        lines.push(format!(
            "settings set target.run-args {}",
            target
                .args
                .iter()
                .map(|argument| lldb_quote(argument))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    let mut environment = case.env.clone();
    environment
        .entry("RUST_BACKTRACE".to_owned())
        .or_insert_with(|| "full".to_owned());
    if let Some(seed) = case.seed {
        environment
            .entry("WORLDTOOLS_SEED".to_owned())
            .or_insert_with(|| seed.to_string());
    }
    for (key, value) in environment {
        lines.push(format!(
            "settings append target.env-vars {}",
            lldb_quote(&format!("{key}={value}"))
        ));
    }
    for breakpoint in &target.breakpoints {
        lines.push(lldb_breakpoint(breakpoint));
    }
    lines.extend([
        "run".to_owned(),
        "thread backtrace all".to_owned(),
        "frame variable".to_owned(),
        "quit".to_owned(),
    ]);
    lines.join("\n") + "\n"
}

fn cdb_script(target: &DebugTarget) -> String {
    let mut lines = vec![".lines -e".to_owned(), "sxe av".to_owned()];
    lines.extend(
        target
            .breakpoints
            .iter()
            .map(|breakpoint| format!("bu {}", cdb_quote(breakpoint))),
    );
    lines.extend([
        "g".to_owned(),
        "~* kb".to_owned(),
        "dv /t /v".to_owned(),
        "q".to_owned(),
    ]);
    lines.join("\n") + "\n"
}

fn lldb_breakpoint(value: &str) -> String {
    if let Some((file, line)) = value.rsplit_once(':')
        && line.parse::<u32>().is_ok()
    {
        return format!("breakpoint set --file {} --line {line}", lldb_quote(file));
    }
    format!("breakpoint set --name {}", lldb_quote(value))
}

fn execute(
    backend: DebugBackend,
    script: &Path,
    case: &DebugCase,
    target: &DebugTarget,
    working_directory: &Path,
) -> Result<()> {
    match backend {
        DebugBackend::Lldb => {
            let executable = doctor::executable_path("lldb")
                .ok_or_else(|| anyhow::anyhow!("lldb is not available on PATH"))?;
            let args = vec!["--source".to_owned(), script.to_string_lossy().into_owned()];
            run_inherited(&executable.to_string_lossy(), &args, working_directory)
        }
        DebugBackend::Cdb => {
            let executable = doctor::executable_path("cdb")
                .ok_or_else(|| anyhow::anyhow!("cdb is not available on PATH"))?;
            let mut args = vec!["-cf".to_owned(), script.to_string_lossy().into_owned()];
            args.push(target.program.to_string_lossy().into_owned());
            args.extend(target.args.iter().cloned());
            let mut environment = case.env.clone();
            environment
                .entry("RUST_BACKTRACE".to_owned())
                .or_insert_with(|| "full".to_owned());
            if let Some(seed) = case.seed {
                environment
                    .entry("WORLDTOOLS_SEED".to_owned())
                    .or_insert_with(|| seed.to_string());
            }
            run_inherited_with_env(
                &executable.to_string_lossy(),
                &args,
                working_directory,
                &environment,
            )
        }
    }
}

fn lldb_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn cdb_quote(value: &str) -> String {
    if value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };

    use crate::case::{DebugCase, DebugTarget};

    use super::{lldb_breakpoint, lldb_script};

    #[test]
    fn emits_source_and_symbol_breakpoints() {
        assert_eq!(
            lldb_breakpoint("src/main.rs:42"),
            "breakpoint set --file \"src/main.rs\" --line 42"
        );
        assert_eq!(
            lldb_breakpoint("worldtools::main"),
            "breakpoint set --name \"worldtools::main\""
        );
    }

    #[test]
    fn script_contains_target_arguments_and_seed() {
        let case = DebugCase {
            name: None,
            command: vec!["ignored".to_owned()],
            working_directory: None,
            expected_exit: 0,
            timeout_seconds: 30,
            repeat: 1,
            seed: Some(9),
            env: BTreeMap::new(),
            debug: None,
        };
        let target = DebugTarget {
            program: PathBuf::from("target/debug/worldtools.exe"),
            args: vec!["--headless".to_owned()],
            breakpoints: vec!["main".to_owned()],
        };
        let script = lldb_script(&case, &target, Path::new("workspace"));

        assert!(script.contains("target/debug/worldtools.exe"));
        assert!(script.contains("--headless"));
        assert!(script.contains("WORLDTOOLS_SEED=9"));
    }
}
