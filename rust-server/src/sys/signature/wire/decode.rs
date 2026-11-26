use super::{Signature, VERSION, tag};
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
        let (bits, used) = take(rest, ty)?;
        values.push(value(ty, bits));
        rest = &rest[used..];
    }
    if !rest.is_empty() {
        return Err("trailing wire bytes".into());
    }
    Ok(values)
}

fn take(bytes: &[u8], ty: u8) -> Result<(u64, usize), String> {
    let width = if ty == b'i' { 4 } else { 8 };
    if bytes.len() < width + 1 || bytes[0] != tag(ty) {
        return Err("truncated or mistyped wire value".into());
    }
    let mut buf = [0u8; 8];
    buf[..width].copy_from_slice(&bytes[1..width + 1]);
    Ok((u64::from_le_bytes(buf), width + 1))
}

fn value(ty: u8, bits: u64) -> Val {
    if ty == b'i' {
        Val::I32(bits as u32 as i32)
    } else {
        Val::I64(bits as i64)
    }
}
