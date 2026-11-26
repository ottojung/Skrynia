use super::{MAX_RAM_BYTES, Ram};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

impl Ram {
    pub fn truncate(&self, size: u64) -> Result<(), &'static str> {
        if size > MAX_RAM_BYTES {
            return Err("nospace");
        }
        let n = usize::try_from(size).map_err(|_| "nospace")?;
        let old = self.slots.read().unwrap().clone();
        let mut next = Vec::new();
        next.try_reserve_exact(n).map_err(|_| "nospace")?;
        for b in old.iter().take(n) {
            next.push(AtomicU8::new(b.load(Ordering::SeqCst)));
        }
        next.resize_with(n, || AtomicU8::new(0));
        *self.slots.write().unwrap() = Arc::new(next);
        Ok(())
    }
}
