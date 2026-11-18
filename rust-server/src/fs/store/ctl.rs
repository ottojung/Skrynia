use super::{Fs, Item};
use std::os::fd::AsRawFd;

impl Fs {
    pub fn close(&self, fd: crate::fs::Fd) -> Result<(), &'static str> {
        self.items
            .lock()
            .unwrap()
            .remove(fd)
            .map(|_| ())
            .ok_or("badfd")
    }
    pub fn sync(&self, fd: crate::fs::Fd) -> Result<(), &'static str> {
        let t = self.items.lock().unwrap();
        match t.get(fd) {
            Some(Item::File(e)) => crate::fs::disk::sync(e),
            Some(Item::Dir(e)) => {
                let rc = unsafe { libc::fsync(e.fd.as_raw_fd()) };
                if rc == 0 { Ok(()) } else { Err("io") }
            }
            None => Err("badfd"),
        }
    }
}
