impl super::Ram {
    pub fn read(&self, off: usize, dst: &mut [u8]) -> Result<usize, &'static str> {
        let b = self.bytes.lock().unwrap();
        if off > b.len() {
            return Err("oob");
        }
        let n = dst.len().min(b.len() - off);
        dst[..n].copy_from_slice(&b[off..off + n]);
        Ok(n)
    }
    pub fn write(&self, off: usize, src: &[u8]) -> Result<usize, &'static str> {
        let mut b = self.bytes.lock().unwrap();
        if off > b.len() {
            return Err("oob");
        }
        let n = src.len().min(b.len() - off);
        b[off..off + n].copy_from_slice(&src[..n]);
        Ok(n)
    }
}
