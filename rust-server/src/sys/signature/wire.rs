mod decode;
mod length;

pub use decode::decode;
pub use length::{COUNT_EXTENDED, VERSION, frame_len};

use super::Signature;
use wasmtime::Val;

pub fn encode(signature: &Signature, args: &[Val]) -> Result<Vec<u8>, String> {
    if args.len() != signature.params().len() {
        return Err("argument count mismatch".into());
    }
    let count = args.len();
    let mut out = Vec::with_capacity(frame_len(signature.params())?);
    out.push(VERSION);
    if count < usize::from(COUNT_EXTENDED) {
        out.push(count as u8);
    } else {
        out.push(COUNT_EXTENDED);
        let wide = u32::try_from(count).map_err(|_| "too many wire values".to_string())?;
        out.extend_from_slice(&wide.to_le_bytes());
    }
    for (ty, value) in signature.params().bytes().zip(args) {
        let (tag, bytes) = scalar(ty, value)?;
        out.push(tag);
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

fn scalar(ty: u8, value: &Val) -> Result<(u8, Vec<u8>), String> {
    match (ty, value) {
        (b'i', Val::I32(v)) => Ok((0x32, v.to_le_bytes().to_vec())),
        (b'l', Val::I64(v)) => Ok((0x40, v.to_le_bytes().to_vec())),
        _ => Err("argument type mismatch".into()),
    }
}

pub fn tag(ty: u8) -> u8 {
    if ty == b'i' { 0x32 } else { 0x40 }
}

pub fn fill(signature: &Signature, values: &[Val], results: &mut [Val]) -> Result<(), String> {
    if values.len() != results.len() || signature.results().len() != results.len() {
        return Err("native result count mismatch".into());
    }
    for (ty, (value, slot)) in signature.results().bytes().zip(values.iter().zip(results)) {
        let valid = matches!((ty, value), (b'i', Val::I32(_)) | (b'l', Val::I64(_)));
        if !valid {
            return Err("native result type mismatch".into());
        }
        *slot = *value;
    }
    Ok(())
}
