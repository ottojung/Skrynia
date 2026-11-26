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

// Largest logical size representable by the signed seek result.
// Physical storage is allocated lazily, not at truncate time.
pub const MAX_RAM_BYTES: u64 = i64::MAX as u64;

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
