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
        let (kind, suffix) = crate::fs::root::select::split(path)?;
        if let Some(want) = root
            && want != kind
        {
            return Err("root");
        }
        let rel = crate::fs::path::clean(suffix)?;
        let e = crate::fs::disk::open_disk(self.root(kind), &rel, mode)?;
        fds.insert(Item::File(e), owner)
    }
}
