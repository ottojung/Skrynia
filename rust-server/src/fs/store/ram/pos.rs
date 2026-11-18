use super::{Fs, Item};

impl Fs {
    pub fn seek_ram(&self, fd: crate::fs::Fd, off: u64) -> Result<(), &'static str> {
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::Ram(e)) => {
                e.pos = off;
                Ok(())
            }
            _ => Err("badfd"),
        }
    }
    pub fn truncate_ram(&self, fd: crate::fs::Fd, size: u64) -> Result<(), &'static str> {
        let t = self.items.lock().unwrap();
        match t.get(fd) {
            Some(Item::Ram(_)) => {
                self.ram.truncate(size as usize);
                Ok(())
            }
            _ => Err("badfd"),
        }
    }
}
