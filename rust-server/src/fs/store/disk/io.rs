use super::{Fs, Item};

impl Fs {
    pub fn read(&self, fd: crate::fs::Fd, dst: &mut [u8]) -> Result<usize, &'static str> {
        if self.is_ram(fd) {
            return self.read_ram(fd, dst);
        }
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::read(e, dst),
            _ => Err("badfd"),
        }
    }
    pub fn write(&self, fd: crate::fs::Fd, src: &[u8]) -> Result<usize, &'static str> {
        if self.is_ram(fd) {
            return self.write_ram(fd, src);
        }
        let mut t = self.items.lock().unwrap();
        match t.get_mut(fd) {
            Some(Item::File(e)) => crate::fs::disk::write(e, src),
            _ => Err("badfd"),
        }
    }
}
