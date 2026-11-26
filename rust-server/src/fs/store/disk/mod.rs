mod byte;
mod io;
mod seek;
pub(super) use super::{Fs, Item, RootKind};

impl Fs {
    pub fn open_disk(
        &self,
        root: Option<RootKind>,
        fds: &mut crate::fs::table::Table<Item>,
        path: &[u8],
        mode: crate::fs::disk::DiskMode,
        owner: u32,
    ) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(path)?;
        let kind = root.unwrap_or_else(|| {
            if mode.write || mode.create || mode.truncate || mode.exclusive {
                RootKind::User
            } else {
                self.pick_read_root(path)
            }
        });
        let e = crate::fs::disk::open_disk(self.root(kind), &rel, mode)?;
        fds.insert(Item::File(e), owner)
    }
}
