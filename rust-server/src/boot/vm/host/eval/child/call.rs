pub fn call_cb(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    cb: wasmtime::Func,
    req: &[u8],
    res_ptr: u32,
    res_cap: u32,
) -> i32 {
    let root_inst = c.data().frames[0].instance;
    let root_mem = match root_inst.get_memory(&mut *c, "memory") {
        Some(m) => m,
        None => return crate::boot::vm::codes::ERR_FAULT,
    };
    let base = root_mem.data_size(&*c) as u32;
    let res_base = match base.checked_add(req.len() as u32) {
        Some(v) => v,
        None => return crate::boot::vm::codes::ERR_LIMIT,
    };
    let need = match (res_base as u64).checked_add(res_cap as u64) {
        Some(v) => v,
        None => return crate::boot::vm::codes::ERR_LIMIT,
    };
    if let Err(e) = super::mem::ensure(c, root_mem, need) {
        return e;
    }
    if crate::boot::vm::mem::write_buf(c, root_mem, base, req).is_err() {
        return crate::boot::vm::codes::ERR_RANGE;
    }
    let cb_t: wasmtime::TypedFunc<(u32, u32, u32, u32), i32> = match cb.typed(&*c) {
        Ok(f) => f,
        Err(_) => return crate::boot::vm::codes::ERR_INVALID,
    };
    let n = match cb_t.call(&mut *c, (base, req.len() as u32, res_base, res_cap)) {
        Ok(n) => n,
        Err(_) => return crate::boot::vm::codes::ERR_FAULT,
    };
    if n < 0 {
        return n;
    }
    super::copy::copy_res(c, root_mem, res_base, res_ptr, n as u32, res_cap)
}
