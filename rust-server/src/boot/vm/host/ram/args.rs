// Fixed ABI pointers/lengths are u64 offsets into the single shared RAM.
// A linear-memory fallback exists ONLY while RAM is uninitialized (len 0),
// for legacy kernel tests not yet ported by K2/K0. Remove after test port.
mod codes;
pub use codes::*;

pub fn read(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    len: u64,
) -> Result<Vec<u8>, i32> {
    let ram = c.data().ram.clone();
    if ram.len() == 0 {
        let p = u32::try_from(ptr).map_err(|_| E_BOUNDS)?;
        let n = u32::try_from(len).map_err(|_| E_BOUNDS)?;
        let mem = crate::boot::vm::mem::get(c)?;
        return crate::boot::vm::mem::read_buf(c, mem, p, n);
    }
    let end = ptr.checked_add(len).ok_or(E_RANGE)?;
    if end > ram.len() {
        return Err(E_BOUNDS);
    }
    let mut buf = vec![0u8; usize::try_from(len).map_err(|_| E_BOUNDS)?];
    ram.read(ptr, &mut buf).map_err(|_| E_BOUNDS)?;
    Ok(buf)
}

pub fn write(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    src: &[u8],
) -> Result<(), i32> {
    let ram = c.data().ram.clone();
    if ram.len() == 0 {
        let m = crate::boot::vm::mem::get(c)?;
        return crate::boot::vm::mem::write_buf(
            c,
            m,
            u32::try_from(ptr).map_err(|_| E_BOUNDS)?,
            src,
        )
        .map_err(|_| E_BOUNDS);
    }
    let end = ptr.checked_add(src.len() as u64).ok_or(E_RANGE)?;
    if end > ram.len() {
        return Err(E_BOUNDS);
    }
    ram.write(ptr, src).map(|_| ()).map_err(|_| E_BOUNDS)
}
