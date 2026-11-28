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
        let root = Self::open_unhardened(path)?;
        root.restrict_permissions()?;
        Ok(root)
    }
    pub(crate) fn open_unhardened(path: &Path) -> Result<Self, String> {
        open::open_root(path)
    }
    pub(crate) fn restrict_permissions(&self) -> Result<(), String> {
        // Harden the opened directory, never a replaceable path.
        if unsafe { libc::fchmod(self.fd(), 0o700) } != 0 {
            return Err(format!(
                "data dir mode: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
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
