pub fn call(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    cb: wasmtime::Func,
    req_ptr: u32,
    req_len: u32,
    res_ptr: u32,
    res_cap: u32,
) -> i32 {
    let mem = match crate::boot::vm::mem::get(c) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let req = match crate::boot::vm::mem::read_buf(c, mem, req_ptr, req_len) {
        Ok(x) => x,
        Err(e) => return e,
    };
    super::call::invoke(c, cb, &req, res_ptr, res_cap)
}
