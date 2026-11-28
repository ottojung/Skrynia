use super::super::Ram;
use super::super::pages::{Pages, SIZE};
use std::sync::Arc;

impl Ram {
    // Keep the shard guard over the atomic accesses, so shrink/tail clearing
    // cannot retire a page while it is being used. No global data lock.
    pub(crate) fn span<T>(
        &self,
        off: u64,
        cap: usize,
        create: bool,
        use_page: impl FnOnce(Option<&Arc<[std::sync::atomic::AtomicU8]>>, usize, usize) -> T,
    ) -> Result<T, &'static str> {
        let index = off / SIZE;
        let within = (off % SIZE) as usize;
        let shard = self.pages.shard(index);
        {
            let entries = shard.read().unwrap();
            let n = self.span_len(off, within, cap)?;
            let page = entries.get(&index);
            if page.is_some() || !create {
                return Ok(use_page(page, within, n));
            }
        }
        let mut entries = shard.write().unwrap();
        let n = self.span_len(off, within, cap)?;
        if !entries.contains_key(&index) {
            entries.try_reserve(1).map_err(|_| "nospace")?;
            entries.insert(index, Pages::allocate()?);
        }
        let page = entries.get(&index);
        Ok(use_page(page, within, n))
    }

    fn span_len(&self, off: u64, within: usize, cap: usize) -> Result<usize, &'static str> {
        let len = self.len();
        if off >= len {
            return Err("oob");
        }
        Ok((len - off).min(SIZE - within as u64).min(cap as u64) as usize)
    }
}
