use super::Ram;
use std::sync::atomic::Ordering;

impl Ram {
    pub fn read(&self, off: u64, dst: &mut [u8]) -> Result<usize, &'static str> {
        let mut pos = 0;
        let limit = self.len().saturating_sub(off).min(dst.len() as u64) as usize;
        while pos < limit {
            let result = self.span(off + pos as u64, limit - pos, false, |page, at, n| {
                let target = &mut dst[pos..pos + n];
                if let Some(page) = page {
                    for (byte, cell) in target.iter_mut().zip(&page[at..at + n]) {
                        *byte = cell.load(Ordering::SeqCst);
                    }
                } else {
                    target.fill(0);
                }
                n
            });
            match result {
                Ok(n) => pos += n,
                Err("oob") => break,
                Err(_) if pos > 0 => break,
                Err(e) => return Err(e),
            }
        }
        Ok(pos)
    }
}
