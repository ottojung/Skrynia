pub struct Ctx {
    pub prev: Option<wasmtime::Instance>,
    pub guard: Option<crate::boot::vm::state::GuestCap>,
    pub go: crate::fs::Fd,
    pub out: crate::fs::store::item::BufferedOutput,
    pub res_ptr: u32,
    pub res_cap: u32,
}
pub fn done(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    x: Ctx,
    result: Result<i32, i32>,
) -> i32 {
    let _ = c.data_mut().fds.remove(x.go);
    let _ = c.data_mut().providers.pop();
    super::super::guest::fds::cleanup(c);
    c.data_mut().guest = x.guard;
    c.data_mut().cur = x.prev;
    let status = match result {
        Ok(0) => 0,
        Ok(n) if n < 0 => n,
        Ok(_) => crate::boot::vm::codes::ERR_INVALID,
        Err(e) => e,
    };
    if status != 0 {
        return status;
    }
    let bytes = x.out.take();
    if bytes.len() as u32 > x.res_cap {
        return crate::boot::vm::codes::ERR_OVERFLOW;
    }
    let mem = match crate::boot::vm::mem::get(c) {
        Ok(m) => m,
        Err(e) => return e,
    };
    if crate::boot::vm::mem::write_buf(c, mem, x.res_ptr, &bytes).is_err() {
        return crate::boot::vm::codes::ERR_RANGE;
    }
    bytes.len() as i32
}
