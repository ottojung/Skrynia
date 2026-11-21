use super::handle::Fd;
use super::store::{Slot, Table};

impl<T> Table<T> {
    pub fn remove_owner(&mut self, owner: u32) {
        if owner == 0 {
            return;
        }
        for i in 0..self.slots.len() {
            let fd = match self.slots.get(i) {
                Some(Slot::Live { tag, owner: o, .. }) if *o == owner => {
                    Some(Fd::new(i as u32, *tag))
                }
                _ => None,
            };
            if let Some(fd) = fd {
                let _ = self.remove(fd);
            }
        }
    }
}
