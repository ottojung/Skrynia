use super::length::{count, frame_len};
use super::{Signature, VERSION, tag};
use wasmtime::Val;

/// Decode a reply against the exact declared result signature. The count and
/// frame length are signature-derived; malformed or non-minimal frames fail
/// before values are decoded.
pub fn decode(signature: &Signature, bytes: &[u8]) -> Result<Vec<Val>, String> {
    if bytes.first().copied() != Some(VERSION) {
        return Err("invalid wire version".into());
    }
    let (count, header) = count(bytes)?;
    if count != signature.results().len() {
        return Err("result count mismatch".into());
    }
    if bytes.len() != frame_len(signature.results())? {
        return Err("wire frame length mismatch".into());
    }
    let mut values = Vec::with_capacity(count);
    let mut at = header;
    for ty in signature.results().bytes() {
        let (bits, used) = take(&bytes[at..], ty)?;
        values.push(value(ty, bits));
        at += used;
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
