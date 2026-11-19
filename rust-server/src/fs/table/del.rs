use super::handle::Fd;
use super::store::{Slot, Table};

impl<T> Table<T> {
    pub fn remove(&mut self, fd: Fd) -> Option<T> {
        let i = fd.idx();
        let tag = match self.slots.get(i) {
            Some(Slot::Live { tag, .. }) if *tag == fd.tag() => *tag,
            _ => return None,
        };
        // Never wrap a generation or encode a valid descriptor as an error.
        let reusable = tag < 0x7fff;
        let slot = if reusable {
            Slot::Free {
                next: self.free,
                tag: tag + 1,
            }
        } else {
            Slot::Retired
        };
        let Slot::Live { val, .. } = std::mem::replace(&mut self.slots[i], slot) else {
            unreachable!()
        };
        if reusable {
            self.free = Some(i);
        }
        Some(val)
    }
}
