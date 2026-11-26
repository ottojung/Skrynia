use super::{Fs, Item};

// Legacy absolute seek used by old-ABI Rust tests until K0/K2 port them.
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
            _ => Err("unsupported"),
        }
    }
}
