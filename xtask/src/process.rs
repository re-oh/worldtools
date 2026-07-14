use std::{
    collections::BTreeMap,
    ffi::OsStr,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use wait_timeout::ChildExt;

#[derive(Clone, Debug)]
pub struct ProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
    pub env: BTreeMap<String, String>,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct ProcessOutput {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration: Duration,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn capture(request: &ProcessRequest) -> Result<ProcessOutput> {
    let started = Instant::now();
    let mut child = Command::new(&request.program)
        .args(&request.args)
        .current_dir(&request.current_dir)
        .envs(&request.env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start {}", request.program))?;

    let stdout = child.stdout.take().context("child stdout was not piped")?;
    let stderr = child.stderr.take().context("child stderr was not piped")?;
    let stdout_reader = read_in_background(stdout);
    let stderr_reader = read_in_background(stderr);

    let status = child
        .wait_timeout(request.timeout)
        .with_context(|| format!("failed while waiting for {}", request.program))?;
    let timed_out = status.is_none();
    let status = if let Some(status) = status {
        status
    } else {
        terminate(&mut child);
        child
            .wait()
            .with_context(|| format!("failed to reap timed-out process {}", request.program))?
    };

    let stdout = join_reader(stdout_reader, "stdout")?;
    let stderr = join_reader(stderr_reader, "stderr")?;
    Ok(ProcessOutput {
        exit_code: status.code(),
        timed_out,
        duration: started.elapsed(),
        stdout,
        stderr,
    })
}

pub fn run_inherited<I, S>(program: &str, args: I, current_dir: &Path) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_inherited_with_env(program, args, current_dir, &BTreeMap::new())
}

pub fn run_inherited_with_env<I, S>(
    program: &str,
    args: I,
    current_dir: &Path,
    env: &BTreeMap<String, String>,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .current_dir(current_dir)
        .envs(env)
        .status()
        .with_context(|| format!("failed to start {program}"))?;
    if !status.success() {
        anyhow::bail!("{program} exited with {status}");
    }
    Ok(())
}

fn read_in_background<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = Vec::new();
        reader.read_to_end(&mut output)?;
        Ok(output)
    })
}

fn join_reader(
    handle: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("{stream} reader thread panicked"))?
        .with_context(|| format!("failed to read child {stream}"))
}

#[cfg(windows)]
fn terminate(child: &mut Child) {
    let _ = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(not(windows))]
fn terminate(child: &mut Child) {
    let _ = child.kill();
}
