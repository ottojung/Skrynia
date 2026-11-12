use super::{disk, staging};
use std::fs;
use std::io;
use std::path::Path;

pub fn commit(dir: &Path, path: &Path, kind: &str, body: &[u8]) -> io::Result<()> {
    if kind == "delete" {
        fs::remove_file(path)?;
    } else {
        let temp = staging::create(dir, body)?;
        let result = if kind == "create" {
            fs::hard_link(&temp, path)
        } else {
            fs::rename(&temp, path)
        };
        let _ = fs::remove_file(temp);
        result?;
    }
    disk::sync_dir(&dir.join("store"))
}
