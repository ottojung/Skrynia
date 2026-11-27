use super::SIZE;
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl super::Ram {
    pub fn write(&self, off: u64, src: &[u8]) -> Result<usize, &'static str> {
        if src.is_empty() {
            return Ok(0);
        }
        let Some(avail) = self.len().checked_sub(off).filter(|v| *v > 0) else {
            return Err("oob");
        };
        let n = core::cmp::min(src.len() as u64, avail) as usize;
        let mut pos = 0usize;
        while pos < n {
            let cur = off + pos as u64;
            let idx = cur / SIZE;
            let within = (cur % SIZE) as usize;
            let chunk = core::cmp::min((SIZE as usize) - within, n - pos);
            let page = match self.page(cur, true) {
                Ok(p) => p.expect("allocated"),
                Err(e) => return if pos == 0 { Err(e) } else { Ok(pos) },
            };
            let entries = self.pages.shard(idx).read().unwrap();
            let now = self.len();
            if cur >= now || !entries.get(&idx).is_some_and(|p| Arc::ptr_eq(p, &page)) {
                return if pos == 0 { Err("oob") } else { Ok(pos) };
            }
            let can = core::cmp::min(chunk as u64, now - cur) as usize;
            for j in 0..can {
                page[within + j].store(src[pos + j], Ordering::SeqCst);
            }
            if can < chunk {
                return Ok(pos + can);
            }
            pos += chunk;
        }
        Ok(n)
    }
}
