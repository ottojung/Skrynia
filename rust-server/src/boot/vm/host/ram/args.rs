// Fixed ABI pointers/lengths are u64 offsets into the single shared RAM.
// Caller WASM linear memory is never used for a fixed-ABI argument.
mod codes;
pub use codes::*;

pub fn read(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    len: u64,
) -> Result<Vec<u8>, i32> {
    let ram = c.data().ram.clone();
    let end = ptr.checked_add(len).ok_or(E_RANGE)?;
    if end > ram.len() {
        return Err(E_BOUNDS);
    }
    let n = usize::try_from(len).map_err(|_| E_BOUNDS)?;
    // Path operations use a bounded host-side staging buffer, not the RAM size.
    if n > crate::boot::limits::MAX_IO as usize {
        return Err(E_BOUNDS);
    }
    let mut buf = Vec::new();
    buf.try_reserve_exact(n).map_err(|_| E_IO)?;
    buf.resize(n, 0);
    if ram.read(ptr, &mut buf).map_err(|_| E_BOUNDS)? != buf.len() {
        return Err(E_BOUNDS);
    }
    Ok(buf)
}

pub fn write(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    src: &[u8],
) -> Result<(), i32> {
    let ram = c.data().ram.clone();
    let end = ptr.checked_add(src.len() as u64).ok_or(E_RANGE)?;
    if end > ram.len() {
        return Err(E_BOUNDS);
    }
    if ram.write(ptr, src).map_err(|_| E_BOUNDS)? != src.len() {
        return Err(E_BOUNDS);
    }
    Ok(())
}
