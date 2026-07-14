use std::path::{Path, PathBuf};

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be directly below the workspace root")
        .to_path_buf()
}

pub fn resolve_case(root: &Path, requested: &Path) -> PathBuf {
    if requested.is_absolute() {
        return requested.to_path_buf();
    }

    let direct = root.join(requested);
    if direct.exists() {
        return direct;
    }

    let mut name = requested.to_path_buf();
    if name.extension().is_none() {
        name.set_extension("toml");
    }
    root.join(".debug").join("cases").join(name)
}
