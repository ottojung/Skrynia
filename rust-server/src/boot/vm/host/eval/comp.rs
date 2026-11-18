pub fn compile(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    code: &[u8],
) -> Result<wasmtime::Module, i32> {
    if let Some(m) = c.data().cache.get(code) {
        return Ok(m);
    }
    let engine = c.data().engine.clone();
    let m =
        wasmtime::Module::new(&engine, code).map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    c.data().cache.put(code, m.clone());
    Ok(m)
}
