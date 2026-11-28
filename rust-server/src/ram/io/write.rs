use super::Ram;
use std::sync::atomic::Ordering;

impl Ram {
    pub fn write(&self, off: u64, src: &[u8]) -> Result<usize, &'static str> {
        if src.is_empty() {
            return Ok(0);
        }
        let limit = self.len().saturating_sub(off).min(src.len() as u64) as usize;
        if limit == 0 {
            return Err("oob");
        }
        let mut pos = 0;
        while pos < limit {
            let result = self.span(off + pos as u64, limit - pos, true, |page, at, n| {
                let page = page.expect("allocated");
                for (cell, byte) in page[at..at + n].iter().zip(&src[pos..pos + n]) {
                    cell.store(*byte, Ordering::SeqCst);
                }
                n
            });
            match result {
                Ok(n) => pos += n,
                Err(_) if pos > 0 => break,
                Err(e) => return Err(e),
            }
        }
        Ok(pos)
    }
}
