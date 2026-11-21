use super::Root;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::fs::OpenOptionsExt;

pub(super) fn open_root(path: &std::path::Path) -> Result<Root, String> {
    let lmeta = std::fs::symlink_metadata(path).map_err(|e| format!("data dir: {e}"))?;
    if lmeta.file_type().is_symlink() {
        return Err("data dir must not be a symlink".into());
    }
    if !lmeta.is_dir() {
        return Err("data dir not directory".into());
    }
    let f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| format!("open data dir: {e}"))?;
    let fd = OwnedFd::from(f);
    let mut st = MaybeUninit::<libc::stat>::uninit();
    let rc = unsafe { libc::fstat(fd.as_raw_fd(), st.as_mut_ptr()) };
    if rc != 0 {
        return Err(format!(
            "stat data dir: {}",
            std::io::Error::last_os_error()
        ));
    }
    let st = unsafe { st.assume_init() };
    Ok(Root {
        fd,
        dev: st.st_dev,
        ino: st.st_ino,
    })
}
