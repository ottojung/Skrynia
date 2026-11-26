use wasmtime::Module;

const ERR_INVAL: i64 = -3;

pub fn validate(module: &Module, bytes: &[u8]) -> Result<bool, i64> {
    let has_start = has_start_section(bytes);
    match module.exports().find(|e| e.name() == "_start") {
        Some(export) => {
            let export_ty = export.ty();
            let ty = export_ty.func().ok_or(ERR_INVAL)?;
            if has_start || ty.params().count() != 0 || ty.results().count() != 0 {
                return Err(ERR_INVAL);
            }
            Ok(true)
        }
        None if has_start => Ok(false),
        None => Err(ERR_INVAL),
    }
}

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
