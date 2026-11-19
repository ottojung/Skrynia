impl super::Ram {
    pub fn cas_u64(&self, off: usize, expected: u64, desired: u64) -> Result<u64, &'static str> {
        if !off.is_multiple_of(8) {
            return Err("unaligned");
        }
        let mut b = self.bytes.lock().unwrap();
        let end = off.checked_add(8).ok_or("oob")?;
        if end > b.len() {
            return Err("oob");
        }
        let old = u64::from_le_bytes(b[off..end].try_into().unwrap());
        if old == expected {
            b[off..end].copy_from_slice(&desired.to_le_bytes());
        }
        Ok(old)
    }
}
