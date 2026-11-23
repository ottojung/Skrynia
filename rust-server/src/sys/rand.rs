pub fn fill_secure(buf: &mut [u8]) -> std::io::Result<()> {
    use std::io::Read;
    let mut f = std::fs::File::open("/dev/urandom")?;
    f.read_exact(buf)
}
pub fn fill_bits(buf: &mut [u8], bits: u32) -> std::io::Result<()> {
    fill_secure(buf)?;
    let rem = bits % 8;
    if rem != 0
        && let Some(last) = buf.last_mut()
    {
        *last &= (1u8 << rem) - 1;
    }
    Ok(())
}
