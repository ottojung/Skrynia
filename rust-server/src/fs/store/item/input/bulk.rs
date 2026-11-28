use super::{Input, transfer};

const CHUNK: usize = 64 * 1024;
impl Input {
    pub fn read(&mut self, dst: &mut [u8]) -> Result<usize, &'static str> {
        if dst.is_empty() {
            return Ok(0);
        }
        let max = self.remaining.min(dst.len() as u64).min(CHUNK as u64) as usize;
        if max == 0 {
            return self.read_local(&mut [0u8; 1]);
        }
        // A native Read may modify scratch bytes beyond its returned prefix.
        // Expose only successful progress to guest memory; reuse bounded storage.
        self.scratch
            .try_reserve(max.saturating_sub(self.scratch.len()))
            .map_err(|_| "nospace")?;
        self.scratch.resize(max, 0);
        let n = transfer::read(
            &mut *self.reader,
            &mut self.remaining,
            &mut self.pos,
            &mut self.scratch,
        )?;
        dst[..n].copy_from_slice(&self.scratch[..n]);
        Ok(n)
    }
}
