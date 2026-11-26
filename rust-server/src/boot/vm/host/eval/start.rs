pub fn has_start_section(bytes: &[u8]) -> bool {
    let mut at = 8;
    while at < bytes.len() {
        let id = bytes[at];
        at += 1;
        let Some((size, used)) = uleb(&bytes[at..]) else {
            return false;
        };
        at += used;
        if id == 8 {
            return true;
        }
        at = match at.checked_add(size) {
            Some(v) if v <= bytes.len() => v,
            _ => return false,
        };
    }
    false
}

fn uleb(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0;
    for (i, byte) in bytes.iter().take(10).enumerate() {
        let part = ((byte & 0x7f) as usize).checked_shl(shift)?;
        value = value.checked_add(part)?;
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
    }
    None
}
