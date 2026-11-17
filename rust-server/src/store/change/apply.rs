use super::Kind;
use crate::Staged;
use std::fs;
use std::io;
use std::path::Path;

pub fn commit(dir: &Path, path: &Path, kind: Kind, staged: Option<&Staged>) -> io::Result<()> {
    let parent = if kind == Kind::Create {
        super::super::paths::ensure_parent(
            dir,
            path.strip_prefix(dir.join("store"))
                .map_err(|_| io::Error::other("invalid store location"))?
                .to_str()
                .ok_or_else(|| io::Error::other("invalid object name"))?,
        )?
    } else {
        super::super::paths::safe_parent(
            dir,
            path.strip_prefix(dir.join("store"))
                .map_err(|_| io::Error::other("invalid store location"))?
                .to_str()
                .ok_or_else(|| io::Error::other("invalid object name"))?,
        )?
    };
    if kind == Kind::Delete {
        fs::remove_file(path)?;
    } else {
        let staged = staged.ok_or_else(|| io::Error::other("staging missing"))?;
        if kind == Kind::Create {
            fs::hard_link(staged.path(), path)?;
        } else {
            fs::rename(staged.path(), path)?;
        }
    }
    crate::sync_dir(&parent)
}
