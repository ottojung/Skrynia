pub fn get_cb(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    cb_idx: i32,
) -> Result<Option<wasmtime::Func>, i32> {
    if cb_idx < 0 {
        return Ok(None);
    }
    let inst = c.data().frames[0].instance;
    let table = inst
        .get_table(&mut *c, "__indirect_function_table")
        .ok_or(crate::boot::vm::codes::ERR_INVALID)?;
    let val = table
        .get(&mut *c, cb_idx as u64)
        .ok_or(crate::boot::vm::codes::ERR_INVALID)?;
    let func = val
        .unwrap_func()
        .ok_or(crate::boot::vm::codes::ERR_INVALID)?;
    func.typed::<(u32, u32, u32, u32), i32>(&*c)
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    Ok(Some(*func))
}
