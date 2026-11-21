fn finish(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    prev: Option<wasmtime::Instance>,
    guard: Option<crate::boot::vm::state::GuestCap>,
) {
    let _ = c.data_mut().providers.pop();
    super::fds::cleanup(c);
    c.data_mut().guest = guard;
    c.data_mut().cur = prev;
}
pub fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    in_fd: i32,
    out_fd: i32,
    cb: Option<wasmtime::Func>,
    eval: bool,
) -> i32 {
    let (prev, guard) = match super::child::setup(c, eval) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let (gi, go) = match super::fds::take(c, in_fd, out_fd) {
        Ok(x) => x,
        Err(e) => {
            let _ = c.data_mut().providers.pop();
            c.data_mut().cur = prev;
            c.data_mut().guest = guard;
            return e;
        }
    };
    let mut linker = wasmtime::Linker::new(&c.data().engine);
    if let Err(e) = super::link::add(&mut linker, cb, eval) {
        finish(c, prev, guard);
        return e;
    }
    let result = super::child::run(c, module, &linker, gi, go);
    finish(c, prev, guard);
    match result {
        Ok(0) => 0,
        Ok(n) if n < 0 => n,
        Ok(_) => crate::boot::vm::codes::ERR_INVALID,
        Err(e) => e,
    }
}
