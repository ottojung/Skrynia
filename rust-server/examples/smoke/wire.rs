use skrynia_server::boot::Kernel;
pub fn request(op: u8, id: &[u8], body: &[u8]) -> Vec<u8> {
    let mut b = vec![1, op];
    if op != 1 {
        b.extend_from_slice(&(id.len() as u16).to_le_bytes());
        b.extend_from_slice(id);
    }
    if op <= 3 {
        b.extend_from_slice(&(body.len() as u32).to_le_bytes());
        b.extend_from_slice(body);
    }
    b
}
pub fn call(k: &Kernel, b: &[u8]) -> Result<Vec<u8>, String> {
    let out = k.invoke(b).map_err(|e| format!("vm error {e}"))?;
    if out.len() < 6 || out[0] != 1 {
        return Err(format!("invalid response {out:?}"));
    }
    let n = u32::from_le_bytes(out[2..6].try_into().unwrap()) as usize;
    if n + 6 != out.len() {
        return Err(format!("wrong length {out:?}"));
    }
    if out[1] != 0 {
        return Err(format!("application error {out:?}"));
    }
    Ok(out[6..].to_vec())
}
