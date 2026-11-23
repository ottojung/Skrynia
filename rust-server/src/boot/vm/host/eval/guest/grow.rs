pub fn ensure(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    mem: wasmtime::Memory,
    need: u64,
) -> Result<(), i32> {
    let have = mem.data_size(&*c) as u64;
    if have >= need {
        return Ok(());
    }
    let grow = (need - have).div_ceil(0x10000);
    mem.grow(&mut *c, grow)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    Ok(())
}
