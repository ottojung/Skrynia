// Legacy 64-bit helper retained only for old-ABI tests. It is a derived
// sequence of single-byte CAS operations, not a fundamental machine CAS.
impl super::Ram {
    pub fn cas_u64(&self, off: usize, expected: u64, desired: u64) -> Result<u64, &'static str> {
        if !off.is_multiple_of(8) {
            return Err("unaligned");
        }
        let base = u64::try_from(off).map_err(|_| "oob")?;
        let mut old = [0u8; 8];
        if self.read(base, &mut old)? != 8 {
            return Err("oob");
        }
        let observed = u64::from_le_bytes(old);
        if observed == expected {
            for (i, b) in desired.to_le_bytes().iter().enumerate() {
                let _ = self.cas1(base + i as u64, old[i], *b);
            }
        }
        Ok(observed)
    }
}
