use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

pub fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

pub fn write_json_atomic(
    directory: &Path,
    prefix: &str,
    value: &impl Serialize,
) -> io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let timestamp = timestamp_millis();
    let final_path = directory.join(format!("{prefix}-{timestamp}.json"));
    let temporary_path = directory.join(format!(".{prefix}-{timestamp}.tmp"));

    {
        let mut writer = BufWriter::new(File::create(&temporary_path)?);
        serde_json::to_writer_pretty(&mut writer, value).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    fs::rename(&temporary_path, &final_path)?;
    Ok(final_path)
}
