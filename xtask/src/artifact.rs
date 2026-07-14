use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::Serialize;

pub fn create_run_directory(workspace: &Path, label: &str) -> Result<PathBuf> {
    let id = unix_millis();
    let safe_label = sanitize(label);
    let path = workspace
        .join(".debug")
        .join("runs")
        .join(format!("{id}-{safe_label}-{}", process::id()));
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create artifact directory {}", path.display()))?;
    Ok(path)
}

pub fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value).context("failed to serialize JSON artifact")?;
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

pub fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sanitize(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    sanitized.trim_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn artifact_labels_are_filesystem_safe() {
        assert_eq!(sanitize("WorldTools: smoke/test"), "WorldTools--smoke-test");
    }
}
