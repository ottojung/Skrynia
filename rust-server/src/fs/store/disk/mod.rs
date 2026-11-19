mod io;
mod seek;
pub(super) use super::{Fs, Item};

impl Fs {
    pub fn open_disk(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        path: &[u8],
        mode: crate::fs::disk::DiskMode,
    ) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(path)?;
        let e = crate::fs::disk::open_disk(&self.root, &rel, mode)?;
        fds.insert(Item::File(e))
    }
}
