use super::{Fs, Item};

impl Fs {
    pub fn seek_rel(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        shift: i64,
    ) -> Result<u64, &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Input(e)) => {
                if shift < 0 {
                    return Err("unsupported");
                }
                e.skip(shift as u64)
            }
            Some(Item::Output(_) | Item::BufferedOutput(_)) => {
                if shift == 0 {
                    Ok(0)
                } else {
                    Err("unsupported")
                }
            }
            Some(Item::Ram(e)) => {
                let next = i128::from(e.pos) + i128::from(shift);
                if next < 0 || next as u64 > self.ram.len() {
                    return Err("overflow");
                }
                e.pos = next as u64;
                Ok(e.pos)
            }
            Some(Item::File(e)) => crate::fs::disk::byte::seek_rel(e, shift),
            Some(Item::Dir(_)) => Err("unsupported"),
            None => Err("badfd"),
        }
    }
    pub fn truncate_fd(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        size: u64,
    ) -> Result<(), &'static str> {
        match fds.get(fd) {
            Some(Item::Input(_) | Item::Output(_)) => Err("perm"),
            Some(Item::Ram(_)) => self.ram.truncate(size),
            Some(Item::File(e)) => crate::fs::disk::truncate(e, size),
            _ => Err("badfd"),
        }
    }
}
