use std::fs::{self, File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;

pub fn acquire(dir: &Path) -> std::io::Result<File> {
    fs::create_dir_all(dir)?;
    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("writer.lock"))?;
    let result = unsafe { libc::flock(fd.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(fd)
}
