mod io;
mod seek;
pub(super) use super::{Fs, Item};

impl Fs {
    pub fn open_disk(
        &self,
        path: &[u8],
        mode: crate::fs::disk::DiskMode,
    ) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(path)?;
        let e = crate::fs::disk::open_disk(&self.root, &rel, mode)?;
        Ok(self.items.lock().unwrap().insert(Item::File(e)))
    }
    pub(super) fn is_ram(&self, fd: crate::fs::Fd) -> bool {
        let t = self.items.lock().unwrap();
        matches!(t.get(fd), Some(Item::Ram(_)))
    }
}
