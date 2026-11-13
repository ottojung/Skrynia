use super::staging;
use std::fs;
use std::io;
use std::path::Path;

pub fn commit(
    dir: &Path,
    path: &Path,
    kind: &str,
    staged: Option<&staging::Staged>,
) -> io::Result<()> {
    if kind == "delete" {
        fs::remove_file(path)?;
    } else {
        let staged = staged.ok_or_else(|| io::Error::other("staging missing"))?;
        if kind == "create" {
            fs::hard_link(staged.path(), path)?;
        } else {
            fs::rename(staged.path(), path)?;
        }
    }
    crate::sync_dir(&dir.join("store"))
}
