use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
mod bytes;
#[cfg(test)]
mod concurrent;
mod io;
mod pages;
#[cfg(test)]
mod regrow;
mod resize;
#[cfg(test)]
mod retained;
#[cfg(test)]
mod tests;

// Existing host capacity bound; independent of execution budgets.
pub const MAX_RAM_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Default)]
pub struct Ram {
    size: AtomicU64,
    pages: pages::Pages,
    resize: Mutex<()>,
}

impl Ram {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> u64 {
        self.size.load(Ordering::SeqCst)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
