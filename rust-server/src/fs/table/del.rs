use super::handle::Fd;
use super::store::{Slot, Table};

impl<T> Table<T> {
    pub fn remove(&mut self, fd: Fd) -> Option<T> {
        let i = fd.idx();
        let tag = match self.slots.get(i) {
            Some(Slot::Live { tag, .. }) if *tag == fd.tag() => *tag,
            _ => return None,
        };
        let next_tag = (tag + 1) & 0xffff;
        let next_tag = if next_tag == 0 { 1 } else { next_tag };
        let Slot::Live { val, .. } = std::mem::replace(
            &mut self.slots[i],
            Slot::Free {
                next: self.free,
                tag: next_tag,
            },
        ) else {
            unreachable!()
        };
        self.free = Some(i);
        Some(val)
    }
}
