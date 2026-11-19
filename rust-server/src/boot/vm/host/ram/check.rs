pub fn bounds(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u32,
    off: u64,
) -> Result<(wasmtime::Memory, usize, usize), i32> {
    let m = crate::boot::vm::mem::get(c)?;
    let end = (ptr as usize)
        .checked_add(8)
        .ok_or(crate::boot::vm::codes::ERR_RANGE)?;
    if end > m.data(&*c).len() {
        return Err(crate::boot::vm::codes::ERR_RANGE);
    }
    let offset = usize::try_from(off).map_err(|_| crate::boot::vm::codes::ERR_RANGE)?;
    Ok((m, end, offset))
}
