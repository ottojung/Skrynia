const ERR_RANGE: i32 = -8;

struct Ram {
    bytes: Vec<u8>,
    touches: u64,
}

impl Ram {
    fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
            touches: 0,
        }
    }
    fn cas_u64(&mut self, off: usize, exp: u64, des: u64, mem: &mut [u8], op: usize) -> i32 {
        if !off.is_multiple_of(8) || off + 8 > self.bytes.len() || op + 8 > mem.len() {
            return ERR_RANGE;
        }
        self.touches += 1;
        let mut cur = [0u8; 8];
        cur.copy_from_slice(&self.bytes[off..off + 8]);
        let old = u64::from_le_bytes(cur);
        if old == exp {
            self.bytes[off..off + 8].copy_from_slice(&des.to_le_bytes());
        }
        mem[op..op + 8].copy_from_slice(&old.to_le_bytes());
        0
    }
}

#[test]
fn cas_unambiguous_status_and_patterns() {
    let mut ram = Ram::new(16);
    let mut mem = [0u8; 16];
    assert_eq!(ram.cas_u64(1, 0, 1, &mut mem, 0), ERR_RANGE);
    assert_eq!(ram.cas_u64(0, 0, u64::MAX, &mut mem, 0), 0);
    assert_eq!(u64::from_le_bytes(mem[0..8].try_into().unwrap()), 0);
    assert_eq!(ram.cas_u64(0, 0, 1, &mut mem, 8), 0);
    assert_eq!(u64::from_le_bytes(mem[8..16].try_into().unwrap()), u64::MAX);
    let hi = 1u64 << 63;
    assert_eq!(ram.cas_u64(0, u64::MAX, hi, &mut mem, 0), 0);
    assert_eq!(u64::from_le_bytes(mem[0..8].try_into().unwrap()), u64::MAX);
    assert_eq!(ram.cas_u64(0, hi, hi, &mut mem, 0), 0);
    assert_eq!(u64::from_le_bytes(mem[0..8].try_into().unwrap()), hi);
}

#[test]
fn ram_not_touched_without_explicit_op() {
    let mut ram = Ram::new(8);
    let before = ram.touches;
    let _ = &ram.bytes;
    assert_eq!(before, ram.touches);
}
