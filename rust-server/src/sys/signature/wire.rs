mod decode;

pub use decode::decode;

use super::Signature;
use wasmtime::Val;

pub const VERSION: u8 = 1;

pub fn encode(signature: &Signature, args: &[Val]) -> Result<Vec<u8>, String> {
    if args.len() != signature.params().len() {
        return Err("argument count mismatch".into());
    }
    let mut out = vec![VERSION, args.len() as u8];
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
