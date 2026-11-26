use super::{Signature, VERSION};
use wasmtime::Val;

pub fn decode(signature: &Signature, bytes: &[u8]) -> Result<Vec<Val>, String> {
    if bytes.len() < 2 || bytes[0] != VERSION {
        return Err("invalid wire version".into());
    }
    let count = bytes[1] as usize;
    if count != signature.results().len() {
        return Err("result count mismatch".into());
    }
    let mut rest = &bytes[2..];
    let mut values = Vec::with_capacity(count);
    for ty in signature.results().bytes() {
        let (tag, bits) = take(rest)?;
        if tag != ty {
            return Err("result type mismatch".into());
        }
        values.push(value(ty, bits));
        rest = &rest[9..];
    }
    if !rest.is_empty() {
        return Err("trailing wire bytes".into());
    }
    Ok(values)
}

fn take(bytes: &[u8]) -> Result<(u8, u64), String> {
    if bytes.len() < 9 {
        return Err("truncated wire value".into());
    }
    let bits: [u8; 8] = bytes[1..9].try_into().map_err(|_| "truncated wire value")?;
    Ok((bytes[0], u64::from_le_bytes(bits)))
}

fn value(ty: u8, bits: u64) -> Val {
    if ty == b'i' {
        Val::I32(bits as i64 as i32)
    } else {
        Val::I64(bits as i64)
    }
}
