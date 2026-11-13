use super::Intent;
use crate::{sync_dir, write_json};
use std::fs;
use std::io;
use std::path::Path;
pub fn run(
    dir: &Path,
    file: &Path,
    intent: &mut Intent,
    notify: impl FnOnce() -> io::Result<()>,
) -> io::Result<bool> {
    if let Err(error) = notify() {
        // An unsent notification must survive subsequent writes to the key.
        // Failure to persist this marker is fatal: callers must fail closed.
        intent.committed = true;
        write_json(file, intent)?;
        eprintln!("Skrynia committed object with pending push recovery: {error}");
        return Ok(true);
    }
    if let Err(error) = fs::remove_file(file).and_then(|_| sync_dir(dir)) {
        // Push is already durably enqueued; a pending intent can be replayed.
        eprintln!("Skrynia notification completed, journal cleanup pending: {error}");
        return Ok(true);
    }
    Ok(false)
}
