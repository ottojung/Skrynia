use super::handle::Fd;

pub(super) const MAX_FDS: usize = 1 << 16;

pub(super) enum Slot<T> {
    Retired,
    Free { next: Option<usize>, tag: u32 },
    Live { tag: u32, owner: u32, val: T },
}

pub struct Table<T> {
    pub(super) slots: Vec<Slot<T>>,
    pub(super) free: Option<usize>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: None,
        }
    }
    pub fn insert(&mut self, val: T, owner: u32) -> Result<Fd, &'static str> {
        if let Some(i) = self.free {
            return Ok(self.reuse(i, val, owner));
        }
        let i = self.slots.len();
        if i >= MAX_FDS {
            return Err("limit");
        }
        self.slots.push(Slot::Live { tag: 1, owner, val });
        Ok(Fd::new(i as u32, 1))
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}
