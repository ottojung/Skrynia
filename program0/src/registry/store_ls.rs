use super::cap::Disk;
use crate::id::ID_LEN;
use crate::store_io::StoreErr;

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

pub fn parse_id_name(name: &str) -> Option<Vec<u8>> {
    let hex = name.strip_suffix(".bin")?;
    if hex.len() != ID_LEN * 2 {
        return None;
    }
    let bytes = hex.as_bytes();
    let mut out = Vec::with_capacity(ID_LEN);
    for i in (0..bytes.len()).step_by(2) {
        out.push((hex_val(bytes[i])? << 4) | hex_val(bytes[i + 1])?);
    }
    Some(out)
}

pub fn list_ids<D: Disk>(disk: &D, dir: &str) -> Result<Vec<Vec<u8>>, StoreErr<D::Err>> {
    let names = disk.list(dir).map_err(StoreErr::Disk)?;
    Ok(names
        .into_iter()
        .filter_map(|n| parse_id_name(&n))
        .collect())
}
