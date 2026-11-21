use std::os::fd::OwnedFd;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub(super) fn open_root(path: &std::path::Path) -> Result<OwnedFd, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("data dir: {e}"))?;
    if !meta.is_dir() {
        return Err("data dir not directory".into());
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("data dir mode: {e}"))?;
    let f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| format!("open data dir: {e}"))?;
    Ok(OwnedFd::from(f))
}
