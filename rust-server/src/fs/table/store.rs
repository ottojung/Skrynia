use super::handle::Fd;

pub(super) enum Slot<T> {
    Free { next: Option<usize>, tag: u32 },
    Live { tag: u32, val: T },
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
    pub fn insert(&mut self, val: T) -> Fd {
        if let Some(i) = self.free {
            return self.reuse(i, val);
        }
        let i = self.slots.len();
        self.slots.push(Slot::Live { tag: 1, val });
        Fd::new(i as u32, 1)
    }
}
