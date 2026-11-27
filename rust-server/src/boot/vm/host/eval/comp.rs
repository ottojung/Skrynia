pub fn compile(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    code: &[u8],
) -> Result<wasmtime::Module, i32> {
    if code.len() > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err(-6);
    }
    let engine = c.data().engine.clone();
    let cache = c.data().cache.clone();
    cache
        .compile(code, || crate::boot::vm::compile::bounded(&engine, code))
        .map_err(|e| if e == "invalid" { -3 } else { -5 })
}
