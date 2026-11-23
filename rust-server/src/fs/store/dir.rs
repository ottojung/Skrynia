use super::{Fs, Item, RootKind};

impl Fs {
    pub fn open_directory(
        &self,
        root: Option<RootKind>,
        fds: &mut crate::fs::table::Table<Item>,
        p: &[u8],
        owner: u32,
    ) -> Result<crate::fs::Fd, &'static str> {
        let rel = crate::fs::path::clean(p)?;
        let kind = root.unwrap_or(self.pick_read_root(p));
        let e = crate::fs::dir::open_dir(self.root(kind), &rel)?;
        fds.insert(Item::Dir(e), owner)
    }
    pub fn next_entry(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        fd: crate::fs::Fd,
        dst: &mut [u8],
    ) -> Result<crate::fs::dir::Next, &'static str> {
        match fds.get_mut(fd) {
            Some(Item::Dir(e)) => crate::fs::dir::next_entry(e, dst),
            _ => Err("badfd"),
        }
    }
}
