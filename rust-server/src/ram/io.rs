// Bulk helpers are loops over byte accesses. They never take a global data mutex.
impl super::Ram {
    pub fn read(&self, off: u64, dst: &mut [u8]) -> Result<usize, &'static str> {
        if off >= self.len() {
            return Ok(0);
        }
        for (i, b) in dst.iter_mut().enumerate() {
            match self.read_byte(off + i as u64) {
                Ok(v) => *b = v,
                Err(_) => return Ok(i),
            }
        }
        Ok(dst.len())
    }
    pub fn write(&self, off: u64, src: &[u8]) -> Result<usize, &'static str> {
        if off > self.len() {
            return Err("oob");
        }
        for (i, b) in src.iter().enumerate() {
            if self.write_byte(off + i as u64, *b).is_err() {
                if i == 0 {
                    return Err("oob");
                }
                return Ok(i);
            }
        }
        Ok(src.len())
    }
}
