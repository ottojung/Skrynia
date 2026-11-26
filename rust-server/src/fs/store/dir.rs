use super::{Fs, Item, RootKind};

impl Fs {
    pub fn open_directory(
        &self,
        root: Option<RootKind>,
        fds: &mut crate::fs::table::Table<Item>,
        p: &[u8],
        owner: u32,
    ) -> Result<crate::fs::Fd, &'static str> {
        let (kind, suffix) = crate::fs::root::select::split(p)?;
        if let Some(want) = root {
            if want != kind {
                return Err("root");
            }
        }
        let e = if suffix == b"." {
            crate::fs::dir::open_root(self.root(kind))?
        } else {
            let rel = crate::fs::path::clean(suffix)?;
            crate::fs::dir::open_dir(self.root(kind), &rel)?
        };
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
