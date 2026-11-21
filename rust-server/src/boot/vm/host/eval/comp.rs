pub fn compile(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    code: &[u8],
) -> Result<wasmtime::Module, i32> {
    if code.len() > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err(crate::boot::vm::codes::ERR_LIMIT);
    }
    if let Some(m) = c.data().cache.get(code) {
        return Ok(m);
    }
    let engine = c.data().engine.clone();
    let m = crate::boot::vm::compile::bounded(&engine, code).map_err(crate::boot::vm::err::map)?;
    c.data().cache.put(code, m.clone());
    Ok(m)
}
