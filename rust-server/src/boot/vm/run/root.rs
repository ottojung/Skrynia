const IN_OFF: u32 = 0;
const OUT_CAP: u32 = 1024 * 1024;

fn ensure(
    store: &mut wasmtime::Store<crate::boot::vm::state::Vm>,
    inst: wasmtime::Instance,
    need: u64,
) -> Result<(), i32> {
    let mem = inst
        .get_memory(&mut *store, "memory")
        .ok_or(crate::boot::vm::codes::ERR_FAULT)?;
    let have = mem.data_size(&*store) as u64;
    if have >= need {
        return Ok(());
    }
    let grow = (need - have).div_ceil(0x10000);
    mem.grow(&mut *store, grow)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    Ok(())
}

pub fn call_root(
    store: &mut wasmtime::Store<crate::boot::vm::state::Vm>,
    body: &[u8],
) -> Result<Vec<u8>, i32> {
    let inst = store.data().frames[0].instance;
    let in_len = body.len() as u32;
    let out_off = in_len;
    ensure(store, inst, (out_off + OUT_CAP) as u64)?;
    let mem = crate::boot::vm::mem::get_in(store, inst)?;
    crate::boot::vm::mem::write_inst(store, mem, IN_OFF, body)?;
    let f: wasmtime::TypedFunc<(u32, u32, u32, u32), i32> = inst
        .get_typed_func(&mut *store, "skrynia_main")
        .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    let n = f
        .call(&mut *store, (IN_OFF, in_len, out_off, OUT_CAP))
        .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    if n < 0 {
        return Err(n);
    }
    let mem = crate::boot::vm::mem::get_in(store, inst)?;
    crate::boot::vm::mem::read_inst(store, mem, out_off, n as u32)
}
