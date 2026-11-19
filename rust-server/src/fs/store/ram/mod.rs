mod io;
mod pos;
use super::item::RamFd;
pub(super) use super::{Fs, Item};

impl Fs {
    pub fn open_ram(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
    ) -> Result<crate::fs::Fd, &'static str> {
        fds.insert(Item::Ram(RamFd { pos: 0 }))
    }
}
