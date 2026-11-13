use std::fs::{self, File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn acquire(dir: &Path) -> std::io::Result<File> {
    fs::create_dir_all(dir)?;
    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(dir.join("writer.lock"))?;
    if !fd.metadata()?.is_file() {
        return Err(std::io::Error::other("writer lock is not a regular file"));
    }
    let result = unsafe { libc::flock(fd.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(fd)
}
