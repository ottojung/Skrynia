pub struct Args {
    pub cp: u32,
    pub cl: u32,
    pub req_ptr: u32,
    pub req_len: u32,
    pub res_ptr: u32,
    pub res_cap: u32,
    pub cb_idx: i32,
}
pub struct Prep {
    pub req: Vec<u8>,
    pub cb: Option<wasmtime::Func>,
    pub module: wasmtime::Module,
}
pub fn load(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    a: &Args,
) -> Result<Prep, i32> {
    if a.cl as usize > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err(crate::boot::vm::codes::ERR_LIMIT);
    }
    if a.req_len > crate::boot::limits::MAX_IO || a.res_cap > crate::boot::limits::MAX_IO {
        return Err(crate::boot::vm::codes::ERR_LIMIT);
    }
    let mem = crate::boot::vm::mem::get(c)?;
    let code = crate::boot::vm::mem::read_buf(c, mem, a.cp, a.cl)?;
    let req = crate::boot::vm::mem::read_buf(c, mem, a.req_ptr, a.req_len)?;
    let cb = super::super::cb::get_cb(c, a.cb_idx)?;
    let module = super::super::comp::compile(c, &code)?;
    if !super::super::check::module_ok(&module, false) {
        return Err(crate::boot::vm::codes::ERR_INVALID);
    }
    Ok(Prep { req, cb, module })
}
