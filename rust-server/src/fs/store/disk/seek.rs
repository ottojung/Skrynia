use super::{Fs, Item};

impl Fs {
    pub fn seek(
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
            Some(Item::File(e)) => crate::fs::disk::seek(e, off),
            _ => Err("badfd"),
        }
    }
    pub fn truncate_fd(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        size: u64,
    ) -> Result<(), &'static str> {
        match fds.get(fd) {
            Some(Item::Ram(_)) => self.ram.truncate(size as usize),
            Some(Item::File(e)) => crate::fs::disk::truncate(e, size),
            _ => Err("badfd"),
        }
    }
}
