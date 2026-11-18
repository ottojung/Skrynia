use super::cap::Ram;

/// A single CAS-protected u64 slot stored explicitly in global RAM.
pub fn load_u64<R: Ram>(ram: &R, off: u64) -> Result<u64, R::Err> {
    let mut buf = [0u8; 8];
    ram.read(off, &mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn store_u64<R: Ram>(ram: &R, off: u64, val: u64) -> Result<(), R::Err> {
    ram.write(off, &val.to_le_bytes())
}

pub fn cas_u64<R: Ram>(ram: &R, off: u64, expect: u64, want: u64) -> Result<u64, R::Err> {
    ram.cas_u64(off, expect, want)
}
