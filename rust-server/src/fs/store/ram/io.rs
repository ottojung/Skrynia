use super::{Fs, Item};

impl Fs {
    pub fn read_ram(&self, fd: crate::fs::Fd, dst: &mut [u8]) -> Result<usize, &'static str> {
        let mut t = self.items.lock().unwrap();
        let pos = match t.get_mut(fd) {
            Some(Item::Ram(e)) => &mut e.pos,
            _ => return Err("badfd"),
        };
        let off = *pos as usize;
        let n = self.ram.read(off, dst)?;
        *pos += n as u64;
        Ok(n)
    }
    pub fn write_ram(&self, fd: crate::fs::Fd, src: &[u8]) -> Result<usize, &'static str> {
        let mut t = self.items.lock().unwrap();
        let pos = match t.get_mut(fd) {
            Some(Item::Ram(e)) => &mut e.pos,
            _ => return Err("badfd"),
        };
        let off = *pos as usize;
        let n = self.ram.write(off, src)?;
        *pos += n as u64;
        Ok(n)
    }
}
