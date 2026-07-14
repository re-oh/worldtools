use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

pub fn write_json<T: Serialize>(value: &T, output: Option<&Path>) -> Result<()> {
    match output {
        Some(path) => {
            let file = File::create(path)
                .with_context(|| format!("failed to create {}", path.display()))?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, value)?;
            writer
                .write_all(b"\n")
                .and_then(|()| writer.flush())
                .with_context(|| format!("failed to finish {}", path.display()))?;
            println!("wrote {}", path.display());
        }
        None => println!("{}", serde_json::to_string_pretty(value)?),
    }
    Ok(())
}
