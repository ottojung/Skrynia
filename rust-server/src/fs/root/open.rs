use super::Root;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::fs::OpenOptionsExt;

pub(super) fn open_root(path: &std::path::Path) -> Result<Root, String> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| format!("data dir: {e}"))?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err("data dir must be a real directory".into());
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| format!("open data dir: {e}"))?;
    let fd = OwnedFd::from(file);
    // Change permissions on the opened directory, never on a replaceable path.
    if unsafe { libc::fchmod(fd.as_raw_fd(), 0o700) } != 0 {
        return Err(format!("data dir mode: {}", std::io::Error::last_os_error()));
    }
    let mut st = MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::fstat(fd.as_raw_fd(), st.as_mut_ptr()) } != 0 {
        return Err(format!("stat data dir: {}", std::io::Error::last_os_error()));
    }
    let st = unsafe { st.assume_init() };
    Ok(Root {
        fd,
        dev: st.st_dev,
        ino: st.st_ino,
    })
}
