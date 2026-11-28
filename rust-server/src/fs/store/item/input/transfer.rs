use std::io::Read;

pub(super) fn read(
    reader: &mut dyn Read,
    remaining: &mut u64,
    pos: &mut u64,
    dst: &mut [u8],
) -> Result<usize, &'static str> {
    if dst.is_empty() {
        return Ok(0);
    }
    let max = (*remaining).min(dst.len() as u64) as usize;
    if max == 0 {
        // Probe only after all accepted bytes have been reported. Never put
        // the rejected overflow byte in the caller's destination.
        return match reader.read(&mut [0u8; 1]) {
            Ok(0) => Ok(0),
            Ok(_) => Err("limit"),
            Err(e) => Err(super::error::read_error(e)),
        };
    }
    let n = reader
        .read(&mut dst[..max])
        .map_err(super::error::read_error)?;
    if n > max {
        return Err("io");
    }
    *remaining -= n as u64;
    *pos += n as u64;
    Ok(n)
}
