pub fn fill_secure(buf: &mut [u8]) -> Result<(), ()> {
    use std::io::Read;
    let mut f = std::fs::File::open("/dev/urandom").map_err(|_| ())?;
    f.read_exact(buf).map_err(|_| ())
}
pub fn fill_bits(buf: &mut [u8], bits: u32) -> Result<(), ()> {
    fill_secure(buf)?;
    let rem = bits % 8;
    if rem != 0 {
        if let Some(last) = buf.last_mut() {
            *last &= (1u8 << rem) - 1;
        }
    }
    Ok(())
}
