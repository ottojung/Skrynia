pub fn call_root(
    store: &mut wasmtime::Store<crate::boot::vm::state::Vm>,
    inst: wasmtime::Instance,
    input_fd: i32,
    output_fd: i32,
) -> Result<(), i32> {
    let f: wasmtime::TypedFunc<(i32, i32), i32> = inst
        .get_typed_func(&mut *store, "skrynia_main")
        .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    let status = f
        .call(&mut *store, (input_fd, output_fd))
        .map_err(crate::boot::vm::err::trap)?;
    match status {
        0 => Ok(()),
        n if n < 0 => Err(n),
        _ => Err(crate::boot::vm::codes::ERR_INVALID),
    }
}
