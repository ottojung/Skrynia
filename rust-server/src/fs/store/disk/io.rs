use super::{Fs, Item};

impl Fs {
    pub fn read(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        dst: &mut [u8],
    ) -> Result<usize, &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Input(e)) => e.read(dst),
            Some(Item::Output(_) | Item::BufferedOutput(_)) => Err("perm"),
            Some(Item::Ram(e)) => {
                let off = e.pos;
                let n = self.ram.read(off, dst)?;
                e.pos += n as u64;
                Ok(n)
            }
            Some(Item::File(e)) => crate::fs::disk::read(e, dst),
            _ => Err("badfd"),
        }
    }
    pub fn write(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        src: &[u8],
    ) -> Result<usize, &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Input(_)) => Err("perm"),
            Some(Item::Output(e)) => e.write(src),
            Some(Item::BufferedOutput(e)) => e.write(src),
            Some(Item::Ram(e)) => {
                let off = e.pos;
                let n = self.ram.write(off, src)?;
                e.pos += n as u64;
                Ok(n)
            }
            Some(Item::File(e)) => crate::fs::disk::write(e, src),
            _ => Err("badfd"),
        }
    }
}
