use crate::id;

pub const REC_VER: u8 = 1;

pub fn encode_record(code: &[u8]) -> Vec<u8> {
    let mut out = vec![REC_VER];
    out.extend_from_slice(code);
    out
}

pub fn decode_record(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.first().copied()? != REC_VER {
        return None;
    }
    Some(bytes[1..].to_vec())
}

pub fn path_for(dir: &str, id: &[u8]) -> Option<String> {
    id::validate(id).ok()?;
    Some(format!(
        "{}/{}.bin",
        dir.trim_end_matches('/'),
        id::to_hex(id)
    ))
}
