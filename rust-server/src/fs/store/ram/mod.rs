mod io;
mod pos;
use super::item::RamFd;
pub(super) use super::{Fs, Item};

impl Fs {
    pub fn open_ram(&self) -> crate::fs::Fd {
        self.items
            .lock()
            .unwrap()
            .insert(Item::Ram(RamFd { pos: 0 }))
    }
}
