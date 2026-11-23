mod io;
mod pos;
use super::item;
use super::{Fs, Item};
pub(super) use item::RamFd;

impl Fs {
    pub fn open_ram(
        &self,
        fds: &mut crate::fs::table::Table<Item>,
        owner: u32,
    ) -> Result<crate::fs::Fd, &'static str> {
        fds.insert(Item::Ram(RamFd { pos: 0 }), owner)
    }
}
