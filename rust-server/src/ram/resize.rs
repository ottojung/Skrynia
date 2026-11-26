use super::{MAX_RAM_BYTES, Ram, pages::SIZE};
use std::sync::atomic::Ordering;

impl Ram {
    pub fn truncate(&self, size: u64) -> Result<(), &'static str> {
        if size > MAX_RAM_BYTES {
            return Err("nospace");
        }
        // Serialize size changes only. Retained pages are never copied or replaced.
        let _guard = self.resize.lock().unwrap();
        let old = self.len();
        if size < old {
            self.size.store(size, Ordering::SeqCst);
            self.pages.retain_below(size.div_ceil(SIZE));
            self.clear_tail(size);
        } else if size > old {
            self.clear_tail(old);
            self.size.store(size, Ordering::SeqCst);
        }
        Ok(())
    }
    fn clear_tail(&self, end: u64) {
        let off = (end % SIZE) as usize;
        if off == 0 {
            return;
        }
        let shard = self.pages.shard(end / SIZE).read().unwrap();
        if let Some(page) = shard.get(&(end / SIZE)) {
            for b in &page[off..] {
                b.store(0, Ordering::SeqCst);
            }
        }
    }
}
