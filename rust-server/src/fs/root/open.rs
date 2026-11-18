use std::fs::File;
use std::os::fd::OwnedFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub(super) fn open_root(path: &Path) -> Result<OwnedFd, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("data dir: {e}"))?;
    if !meta.is_dir() {
        return Err("data dir not directory".into());
    }
    let f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| format!("open data dir: {e}"))?;
    Ok(OwnedFd::from(f))
}
