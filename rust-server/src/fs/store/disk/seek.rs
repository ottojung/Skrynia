use super::{Fs, Item};

impl Fs {
    pub fn seek(&self, fd: crate::fs::Fd, off: u64) -> Result<(), &'static str> {
        if self.is_ram(fd) {
            return self.seek_ram(fd, off);
        }
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::seek(e, off),
            _ => Err("badfd"),
        }
    }
    pub fn truncate_fd(&self, fd: crate::fs::Fd, size: u64) -> Result<(), &'static str> {
        if self.is_ram(fd) {
            return self.truncate_ram(fd, size);
        }
        let t = self.items.lock().unwrap();
        match t.get(fd) {
            Some(Item::File(e)) => crate::fs::disk::truncate(e, size),
            _ => Err("badfd"),
        }
    }
}
