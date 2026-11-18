const ERR_RANGE: i32 = -8;

struct Ram {
    bytes: Vec<u8>,
}

impl Ram {
    fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }
    fn cas_u64(&mut self, off: usize, expected: u64, desired: u64) -> Result<u64, i32> {
        if !off.is_multiple_of(8) || off + 8 > self.bytes.len() {
            return Err(ERR_RANGE);
        }
        let mut cur = [0u8; 8];
        cur.copy_from_slice(&self.bytes[off..off + 8]);
        let old = u64::from_le_bytes(cur);
        if old == expected {
            self.bytes[off..off + 8].copy_from_slice(&desired.to_le_bytes());
        }
        Ok(old)
    }
}

#[test]
fn cas_is_atomic_and_checks_alignment_and_overlap_cases() {
    let mut ram = Ram::new(16);
    assert_eq!(ram.cas_u64(1, 0, 1), Err(ERR_RANGE));
    assert_eq!(ram.cas_u64(0, 0, 7).unwrap(), 0);
    ram.bytes[0] = 3;
    assert_eq!(ram.cas_u64(0, 7, 1).unwrap(), 3);
}
