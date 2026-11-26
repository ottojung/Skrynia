use super::handle::Fd;
use super::store::{Slot, Table};

impl<T> Table<T> {
    pub(super) fn reuse(&mut self, i: usize, val: T) -> Fd {
        let tag = match self.slots[i] {
            Slot::Free { next, tag } => {
                self.free = next;
                tag
            }
            Slot::Live { .. } | Slot::Retired => unreachable!(),
        };
        self.slots[i] = Slot::Live { tag, val };
        Fd::new(i as u32, tag)
    }
    pub fn get(&self, fd: Fd) -> Option<&T> {
        match self.slots.get(fd.idx()) {
            Some(Slot::Live { tag, val, .. }) if *tag == fd.tag() => Some(val),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, fd: Fd) -> Option<&mut T> {
        match self.slots.get_mut(fd.idx()) {
            Some(Slot::Live { tag, val, .. }) if *tag == fd.tag() => Some(val),
            _ => None,
        }
    }
}
