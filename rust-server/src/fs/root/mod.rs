use std::os::fd::OwnedFd;
use std::path::Path;
mod open;

pub struct Root {
    fd: OwnedFd,
}

impl Root {
    pub fn open(path: &Path) -> Result<Self, String> {
        open::open_root(path).map(|fd| Self { fd })
    }
    pub fn fd(&self) -> std::os::fd::RawFd {
        use std::os::fd::AsRawFd;
        self.fd.as_raw_fd()
    }
}
