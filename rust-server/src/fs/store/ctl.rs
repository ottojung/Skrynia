use super::{Fs, Item};
use std::os::fd::AsRawFd;

impl Fs {
    pub fn close(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
    ) -> Result<(), &'static str> {
        fds.remove(fd).map(|_| ()).ok_or("badfd")
    }
    pub fn sync(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
    ) -> Result<(), &'static str> {
        match fds.get(fd) {
            Some(Item::File(e)) => crate::fs::disk::sync(e),
            Some(Item::Dir(e)) => {
                let rc = unsafe { libc::fsync(e.fd.as_raw_fd()) };
                if rc == 0 { Ok(()) } else { Err("io") }
            }
            Some(Item::Ram(_)) => Ok(()),
            Some(Item::Input(_)) => Err("perm"),
            None => Err("badfd"),
        }
    }
}
