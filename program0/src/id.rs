pub const ID_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdErr {
    Len,
}

pub fn validate(id: &[u8]) -> Result<(), IdErr> {
    if id.len() != ID_LEN {
        return Err(IdErr::Len);
    }
    Ok(())
}

pub fn to_hex(id: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(id.len() * 2);
    for &b in id {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
