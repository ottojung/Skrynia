use super::SIZE;
use std::sync::atomic::Ordering;

impl super::Ram {
    pub fn read(&self, off: u64, dst: &mut [u8]) -> Result<usize, &'static str> {
        let Some(avail) = self.len().checked_sub(off) else {
            return Ok(0);
        };
        let n = core::cmp::min(dst.len() as u64, avail) as usize;
        let mut pos = 0usize;
        while pos < n {
            let cur = off + pos as u64;
            let idx = cur / SIZE;
            let within = (cur % SIZE) as usize;
            let chunk = core::cmp::min((SIZE as usize) - within, n - pos);
            let entries = self.pages.shard(idx).read().unwrap();
            let now = self.len();
            if cur >= now {
                return Ok(pos);
            }
            let can = core::cmp::min(chunk as u64, now - cur) as usize;
            if let Some(page) = entries.get(&idx) {
                for j in 0..can {
                    dst[pos + j] = page[within + j].load(Ordering::SeqCst);
                }
            } else {
                dst[pos..pos + can].fill(0);
            }
            if can < chunk {
                return Ok(pos + can);
            }
            pos += chunk;
        }
        Ok(n)
    }
}
