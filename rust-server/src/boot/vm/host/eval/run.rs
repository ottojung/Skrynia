pub fn eval(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    cp: u32,
    cl: u32,
    in_fd: i32,
    out_fd: i32,
    cb_idx: i32,
    child_eval: bool,
) -> i32 {
    if cl as usize > crate::boot::vm::state::MAX_MODULE_BYTES {
        return crate::boot::vm::codes::ERR_LIMIT;
    }
    let mem = match crate::boot::vm::mem::get(c) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let code = match crate::boot::vm::mem::read_buf(c, mem, cp, cl) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let cb = match super::cb::get_cb(c, cb_idx) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let module = match super::comp::compile(c, &code) {
        Ok(m) => m,
        Err(e) => return e,
    };
    if !super::check::module_ok(&module, child_eval) {
        return crate::boot::vm::codes::ERR_INVALID;
    }
    super::guest::run(c, &module, in_fd, out_fd, cb, child_eval)
}
