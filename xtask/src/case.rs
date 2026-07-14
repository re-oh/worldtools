use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DebugCase {
    #[serde(default)]
    pub name: Option<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub working_directory: Option<PathBuf>,
    #[serde(default)]
    pub expected_exit: i32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_repeat")]
    pub repeat: usize,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub debug: Option<DebugTarget>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DebugTarget {
    pub program: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub breakpoints: Vec<String>,
}

const fn default_timeout() -> u64 {
    30
}

const fn default_repeat() -> usize {
    1
}

impl DebugCase {
    pub fn load(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read debug case {}", path.display()))?;
        let case: Self = toml::from_str(&source)
            .with_context(|| format!("failed to parse debug case {}", path.display()))?;
        case.validate()?;
        Ok(case)
    }

    pub fn validate(&self) -> Result<()> {
        if self.command.is_empty() || self.command[0].trim().is_empty() {
            bail!("case command must contain a program");
        }
        if self.repeat == 0 {
            bail!("case repeat must be greater than zero");
        }
        if self.timeout_seconds == 0 {
            bail!("case timeout_seconds must be greater than zero");
        }
        if self
            .env
            .keys()
            .any(|key| key.is_empty() || key.contains('='))
        {
            bail!("environment keys must be nonempty and cannot contain '='");
        }
        Ok(())
    }

    pub fn working_directory(&self, workspace: &Path) -> PathBuf {
        self.working_directory
            .as_ref()
            .map_or_else(|| workspace.to_path_buf(), |path| workspace.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::DebugCase;

    #[test]
    fn defaults_create_a_single_bounded_attempt() {
        let case: DebugCase =
            toml::from_str("command = ['cargo', '--version']").expect("minimal case should parse");

        assert_eq!(case.expected_exit, 0);
        assert_eq!(case.repeat, 1);
        assert_eq!(case.timeout_seconds, 30);
        case.validate().expect("minimal case should be valid");
    }

    #[test]
    fn rejects_zero_repetitions() {
        let case: DebugCase =
            toml::from_str("command = ['cargo']\nrepeat = 0").expect("case should parse");

        assert!(case.validate().is_err());
    }
}
