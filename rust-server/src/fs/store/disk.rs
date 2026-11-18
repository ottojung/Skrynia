use super::{Fs, Item};

impl Fs {
    pub fn open_disk(
        &self,
        path: &[u8],
        mode: crate::fs::disk::DiskMode,
    ) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(path)?;
        let e = crate::fs::disk::open_disk(&self.root, &rel, mode)?;
        Ok(self.items.lock().unwrap().insert(Item::File(e)))
    }
    pub fn read(&self, fd: crate::fs::Fd, dst: &mut [u8]) -> Result<usize, &'static str> {
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::read(e, dst),
            _ => Err("badfd"),
        }
    }
    pub fn write(&self, fd: crate::fs::Fd, src: &[u8]) -> Result<usize, &'static str> {
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::write(e, src),
            _ => Err("badfd"),
        }
    }
    pub fn seek(&self, fd: crate::fs::Fd, off: u64) -> Result<(), &'static str> {
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::seek(e, off),
            _ => Err("badfd"),
        }
    }
    pub fn truncate_fd(&self, fd: crate::fs::Fd, size: u64) -> Result<(), &'static str> {
        let t = self.items.lock().unwrap();
        match t.get(fd) {
            Some(Item::File(e)) => crate::fs::disk::truncate(e, size),
            _ => Err("badfd"),
        }
    }
}
