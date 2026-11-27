use std::os::fd::OwnedFd;
use std::path::Path;
mod open;
pub mod parent;
pub mod rel;
pub mod select;
pub mod walk;

pub struct Root {
    fd: OwnedFd,
    dev: libc::dev_t,
    ino: libc::ino_t,
}

impl Root {
    pub fn open(path: &Path) -> Result<Self, String> {
        open::open_root(path)
    }
    pub fn fd(&self) -> std::os::fd::RawFd {
        use std::os::fd::AsRawFd;
        self.fd.as_raw_fd()
    }
    pub fn same_fs(&self, other: &Self) -> bool {
        self.dev == other.dev && self.ino == other.ino
    }
    pub fn id(&self) -> (libc::dev_t, libc::ino_t) {
        (self.dev, self.ino)
    }
}
