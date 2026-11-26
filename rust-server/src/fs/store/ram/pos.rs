use super::{Fs, Item};

impl Fs {
    pub fn seek_ram(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        off: u64,
    ) -> Result<(), &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Ram(e)) => {
                e.pos = off;
                Ok(())
            }
            _ => Err("badfd"),
        }
    }
    pub fn truncate_ram(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        size: u64,
    ) -> Result<(), &'static str> {
        match fds.get(fd) {
            Some(Item::Ram(_)) => self.ram.truncate(size),
            _ => Err("badfd"),
        }
    }
}
