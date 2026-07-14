use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::{
    artifact,
    case::DebugCase,
    cli::CaseArgs,
    doctor,
    process::{ProcessRequest, capture},
    workspace,
};

#[derive(Debug, Serialize)]
struct RunMetadata {
    schema_version: u32,
    case_path: PathBuf,
    command: Vec<String>,
    working_directory: PathBuf,
    expected_exit: i32,
    timeout_seconds: u64,
    repeat: usize,
    seed: Option<u64>,
    environment_keys: Vec<String>,
    capture_environment: bool,
    started_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct RunResult {
    success: bool,
    expected_attempts: usize,
    attempts: Vec<AttemptResult>,
}

#[derive(Debug, Serialize)]
struct AttemptResult {
    attempt: usize,
    success: bool,
    exit_code: Option<i32>,
    expected_exit: i32,
    timed_out: bool,
    duration_ms: u128,
    stdout: String,
    stderr: String,
    error: Option<String>,
}

pub fn run(workspace_root: &Path, arguments: &CaseArgs, capture_environment: bool) -> Result<()> {
    let case_path = workspace::resolve_case(workspace_root, &arguments.case);
    let case = DebugCase::load(&case_path)?;
    let label = case
        .name
        .as_deref()
        .or_else(|| case_path.file_stem().and_then(|value| value.to_str()))
        .unwrap_or("case");
    let run_directory = artifact::create_run_directory(workspace_root, label)?;
    let current_dir = case.working_directory(workspace_root);
    let env = controlled_environment(&case);

    let metadata = RunMetadata {
        schema_version: 1,
        case_path: case_path.clone(),
        command: case.command.clone(),
        working_directory: current_dir.clone(),
        expected_exit: case.expected_exit,
        timeout_seconds: case.timeout_seconds,
        repeat: case.repeat,
        seed: case.seed,
        environment_keys: env.keys().cloned().collect(),
        capture_environment,
        started_unix_ms: artifact::unix_millis(),
    };
    artifact::write_json(&run_directory.join("meta.json"), &metadata)?;
    if capture_environment {
        artifact::write_json(
            &run_directory.join("doctor.json"),
            &doctor::collect(workspace_root),
        )?;
    }

    let mut result = RunResult {
        success: false,
        expected_attempts: case.repeat,
        attempts: Vec::with_capacity(case.repeat),
    };
    let mut combined_stdout = Vec::new();
    let mut combined_stderr = Vec::new();

    for attempt in 1..=case.repeat {
        println!(
            "repro {attempt}/{}: {}",
            case.repeat,
            format_command(&case.command)
        );
        let stdout_name = format!("attempt-{attempt:02}-stdout.log");
        let stderr_name = format!("attempt-{attempt:02}-stderr.log");
        let attempt_result = execute_attempt(
            &case,
            &current_dir,
            &env,
            &run_directory,
            attempt,
            stdout_name,
            stderr_name,
            &mut combined_stdout,
            &mut combined_stderr,
        )?;
        result.attempts.push(attempt_result);
        result.success = result.attempts.len() == result.expected_attempts
            && result.attempts.iter().all(|attempt| attempt.success);
        artifact::write_json(&run_directory.join("result.json"), &result)?;
    }

    artifact::write(&run_directory.join("stdout.log"), &combined_stdout)?;
    artifact::write(&run_directory.join("stderr.log"), &combined_stderr)?;
    println!("artifacts: {}", run_directory.display());

    if !result.success {
        bail!("reproduction did not meet its expected result");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn execute_attempt(
    case: &DebugCase,
    current_dir: &Path,
    env: &BTreeMap<String, String>,
    run_directory: &Path,
    attempt: usize,
    stdout_name: String,
    stderr_name: String,
    combined_stdout: &mut Vec<u8>,
    combined_stderr: &mut Vec<u8>,
) -> Result<AttemptResult> {
    let request = ProcessRequest {
        program: case.command[0].clone(),
        args: case.command[1..].to_vec(),
        current_dir: current_dir.to_path_buf(),
        env: env.clone(),
        timeout: Duration::from_secs(case.timeout_seconds),
    };

    let output = match capture(&request) {
        Ok(output) => output,
        Err(error) => {
            let message = format!("{error:#}");
            artifact::write(&run_directory.join(&stdout_name), b"")?;
            artifact::write(&run_directory.join(&stderr_name), message.as_bytes())?;
            append_log(combined_stderr, attempt, message.as_bytes());
            return Ok(AttemptResult {
                attempt,
                success: false,
                exit_code: None,
                expected_exit: case.expected_exit,
                timed_out: false,
                duration_ms: 0,
                stdout: stdout_name,
                stderr: stderr_name,
                error: Some(message),
            });
        }
    };

    artifact::write(&run_directory.join(&stdout_name), &output.stdout)?;
    artifact::write(&run_directory.join(&stderr_name), &output.stderr)?;
    append_log(combined_stdout, attempt, &output.stdout);
    append_log(combined_stderr, attempt, &output.stderr);
    let success = !output.timed_out && output.exit_code == Some(case.expected_exit);
    Ok(AttemptResult {
        attempt,
        success,
        exit_code: output.exit_code,
        expected_exit: case.expected_exit,
        timed_out: output.timed_out,
        duration_ms: output.duration.as_millis(),
        stdout: stdout_name,
        stderr: stderr_name,
        error: None,
    })
}

fn controlled_environment(case: &DebugCase) -> BTreeMap<String, String> {
    let mut env = case.env.clone();
    env.entry("RUST_BACKTRACE".to_owned())
        .or_insert_with(|| "full".to_owned());
    if let Some(seed) = case.seed {
        env.entry("WORLDTOOLS_SEED".to_owned())
            .or_insert_with(|| seed.to_string());
    }
    env
}

fn append_log(destination: &mut Vec<u8>, attempt: usize, contents: &[u8]) {
    destination.extend_from_slice(format!("\n===== attempt {attempt:02} =====\n").as_bytes());
    destination.extend_from_slice(contents);
    if !contents.ends_with(b"\n") {
        destination.push(b'\n');
    }
}

fn format_command(command: &[String]) -> String {
    command
        .iter()
        .map(|part| {
            if part.contains(' ') {
                format!("\"{part}\"")
            } else {
                part.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::case::DebugCase;

    use super::{controlled_environment, format_command};

    #[test]
    fn injects_seed_without_overwriting_case_environment() {
        let case = DebugCase {
            name: None,
            command: vec!["cargo".to_owned()],
            working_directory: None,
            expected_exit: 0,
            timeout_seconds: 1,
            repeat: 1,
            seed: Some(42),
            env: BTreeMap::from([("RUST_BACKTRACE".to_owned(), "1".to_owned())]),
            debug: None,
        };
        let env = controlled_environment(&case);

        assert_eq!(env.get("WORLDTOOLS_SEED").map(String::as_str), Some("42"));
        assert_eq!(env.get("RUST_BACKTRACE").map(String::as_str), Some("1"));
    }

    #[test]
    fn formats_readable_commands() {
        assert_eq!(
            format_command(&["cargo".to_owned(), "hello world".to_owned()]),
            "cargo \"hello world\""
        );
    }
}
