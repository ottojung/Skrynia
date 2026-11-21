pub fn buf(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    mem: wasmtime::Memory,
    req: &[u8],
    res_cap: u32,
) -> Result<(u32, u32), i32> {
    let base = mem.data_size(&*c) as u32;
    let res = base
        .checked_add(req.len() as u32)
        .ok_or(crate::boot::vm::codes::ERR_LIMIT)?;
    let need = (res as u64)
        .checked_add(res_cap as u64)
        .ok_or(crate::boot::vm::codes::ERR_LIMIT)?;
    super::grow::ensure(c, mem, need)?;
    crate::boot::vm::mem::write_buf(c, mem, base, req)?;
    Ok((base, res))
}
