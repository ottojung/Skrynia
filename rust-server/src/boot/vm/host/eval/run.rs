pub struct Args {
    pub code_ptr: u32,
    pub code_len: u32,
    pub in_ptr: u32,
    pub in_len: u32,
    pub cb_idx: i32,
    pub out_ptr: u32,
    pub out_cap: u32,
}

pub fn eval(c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>, a: Args) -> i32 {
    let mem = match crate::boot::vm::mem::get(c) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let code = match crate::boot::vm::mem::read_buf(c, mem, a.code_ptr, a.code_len) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let input = match crate::boot::vm::mem::read_buf(c, mem, a.in_ptr, a.in_len) {
        Ok(x) => x,
        Err(e) => return e,
    };
    if let Some(f) = c.data_mut().frames.first_mut() {
        f.last_overflow = -1;
    }
    let cb = match super::cb::get_cb(c, a.cb_idx) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let module = match super::comp::compile(c, &code) {
        Ok(m) => m,
        Err(e) => return e,
    };
    if !super::check::module_ok(&module) {
        return crate::boot::vm::codes::ERR_INVALID;
    }
    super::child::run_child(c, &module, &input, cb, a.out_ptr, a.out_cap)
}
