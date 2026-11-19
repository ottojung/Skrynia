pub fn finish(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    child_mem: wasmtime::Memory,
    n: i32,
    out_off: u32,
    out_ptr: u32,
    out_cap: u32,
) -> Result<i32, i32> {
    if n < 0 {
        return Err(n);
    }
    let n = n as u32;
    if n > out_cap || n > 1024 * 1024 {
        c.data_mut().root_mut().last_overflow = n as i32;
        return Err(crate::boot::vm::codes::ERR_OVERFLOW);
    }
    c.data_mut().root_mut().last_overflow = -1;
    let out_bytes = crate::boot::vm::mem::read_buf(c, child_mem, out_off, n)?;
    let root_mem = crate::boot::vm::mem::get(c)?;
    crate::boot::vm::mem::write_buf(c, root_mem, out_ptr, &out_bytes)?;
    Ok(n as i32)
}
