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
    let base = 1u32 << 20;
    let res_base = base + req.len() as u32;
    if let Err(e) = super::mem::ensure(c, root_mem, (res_base + res_cap) as u64) {
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
