use super::disk::sync_dir;
use super::intent::Intent;
use std::fs;
use std::io;
use std::path::Path;

pub fn recover(root: &Path, mut notify: impl FnMut(&Intent) -> io::Result<()>) -> io::Result<()> {
    let dir = root.join("state/store-transactions");
    let mut removed = false;
    for item in fs::read_dir(&dir)? {
        let file = item?.path();
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = name.strip_suffix(".json").unwrap_or("");
        if !crate::valid_identifier(stem) {
            return Err(io::Error::other("invalid transaction filename"));
        }
        let intent: Intent = serde_json::from_slice(&fs::read(&file)?)?;
        if !intent.valid() {
            return Err(io::Error::other("invalid transaction record"));
        }
        if intent.committed || intent.applied(root)? {
            notify(&intent)?;
        }
        fs::remove_file(file)?;
        removed = true;
    }
    if removed { sync_dir(&dir) } else { Ok(()) }
}

#[cfg(test)]
mod tests;
