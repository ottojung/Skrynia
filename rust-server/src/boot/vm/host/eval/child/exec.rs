pub fn run_inner(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    input: &[u8],
    cb: Option<wasmtime::Func>,
    out_ptr: u32,
    out_cap: u32,
) -> Result<i32, i32> {
    let engine = c.data().engine.clone();
    let mut linker = wasmtime::Linker::new(&engine);
    super::link::link_invoke(&mut linker, cb)?;
    let inst = linker
        .instantiate(&mut *c, module)
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    let child_mem = inst
        .get_memory(&mut *c, "memory")
        .ok_or(crate::boot::vm::codes::ERR_FAULT)?;
    let in_off = 0u32;
    let out_off = input.len() as u32;
    let child_cap = (out_off).max(65536);
    super::mem::ensure(c, child_mem, (out_off + child_cap) as u64)?;
    crate::boot::vm::mem::write_buf(c, child_mem, in_off, input)?;
    let main: wasmtime::TypedFunc<(u32, u32, u32, u32), i32> = inst
        .get_typed_func(&mut *c, "skrynia_main")
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    let n = main
        .call(&mut *c, (in_off, input.len() as u32, out_off, child_cap))
        .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    super::fin::finish(c, child_mem, n, out_off, out_ptr, out_cap)
}
