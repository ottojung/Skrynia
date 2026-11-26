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
        let (tag, bits) = scalar(ty, value)?;
        out.push(tag);
        out.extend_from_slice(&bits.to_le_bytes());
    }
    Ok(out)
}

fn scalar(ty: u8, value: &Val) -> Result<(u8, u64), String> {
    let bits = match (ty, value) {
        (b'i', Val::I32(v)) => *v as i64 as u64,
        (b'l', Val::I64(v)) => *v as u64,
        _ => return Err("argument type mismatch".into()),
    };
    Ok((ty, bits))
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
