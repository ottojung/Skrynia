pub fn copy_res(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    root_mem: wasmtime::Memory,
    res_base: u32,
    res_ptr: u32,
    n: u32,
    res_cap: u32,
) -> i32 {
    let copy_n = n.min(res_cap);
    let resp = match crate::boot::vm::mem::read_buf(c, root_mem, res_base, copy_n) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let child_mem = match crate::boot::vm::mem::get(c) {
        Ok(m) => m,
        Err(e) => return e,
    };
    if crate::boot::vm::mem::write_buf(c, child_mem, res_ptr, &resp).is_err() {
        return crate::boot::vm::codes::ERR_RANGE;
    }
    n as i32
}
