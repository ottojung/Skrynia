pub const COUNT_EXTENDED: u8 = 0xff;
pub const VERSION: u8 = 1;
/// Practical ceiling for one dynamically allocated frame (64 MiB of values).
pub const MAX_FRAME: usize = 64 * 1024 * 1024;

/// Exact encoded size of one signature's values, including the count field.
pub fn frame_len(types: &str) -> Result<usize, String> {
    let header: usize = if types.len() < usize::from(COUNT_EXTENDED) {
        2
    } else {
        6
    };
    let values = types.bytes().try_fold(0usize, |sum, ty| {
        let width = match ty {
            b'i' => 5,
            b'l' => 9,
            _ => return Err("invalid integer type".to_string()),
        };
        sum.checked_add(width)
            .ok_or_else(|| "wire length overflow".to_string())
    })?;
    let total = header
        .checked_add(values)
        .ok_or_else(|| "wire length overflow".to_string())?;
    if total > MAX_FRAME {
        return Err("wire frame too large".to_string());
    }
    Ok(total)
}

/// Read the count field. Short counts stay literal; `0xff` introduces a
/// checked little-endian `u32` count, which must be at least 255.
pub fn count(bytes: &[u8]) -> Result<(usize, usize), String> {
    let first = *bytes.get(1).ok_or("truncated wire count")?;
    if first != COUNT_EXTENDED {
        return Ok((first as usize, 2));
    }
    let tail = bytes.get(2..6).ok_or("truncated extended wire count")?;
    let array: [u8; 4] = tail
        .try_into()
        .map_err(|_| "truncated extended wire count")?;
    let wide = u32::from_le_bytes(array) as usize;
    if wide < usize::from(COUNT_EXTENDED) {
        return Err("non-minimal extended wire count".to_string());
    }
    Ok((wide, 6))
}
