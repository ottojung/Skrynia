fn finish(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    prev: Option<wasmtime::Instance>,
    guard: Option<crate::boot::vm::state::GuestCap>,
    auth: crate::boot::vm::cap::FsAuth,
    delegate: crate::boot::vm::cap::FsAuth,
    ret: i32,
) -> i32 {
    c.data_mut().pop_provider();
    super::fds::cleanup(c);
    c.data_mut().guest = guard;
    c.data_mut().cur = prev;
    c.data_mut().fs_auth = auth;
    c.data_mut().fs_delegate = delegate;
    ret
}
pub fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    in_fd: i32,
    out_fd: i32,
    cb: Option<wasmtime::Func>,
    eval: bool,
) -> i32 {
    let (prev, guard, auth, delegate) = match super::child::setup(c, eval) {
        Ok(x) => x,
        Err(e) => return e,
    };
    if eval {
        c.data_mut().fs_auth = delegate.clone();
        c.data_mut().fs_delegate = crate::boot::vm::cap::FsAuth::Empty;
    }
    let (gi, go) = match super::fds::take(c, in_fd, out_fd) {
        Ok(x) => x,
        Err(e) => return finish(c, prev, guard, auth, delegate, e),
    };
    let prov = c.data().providers.last().cloned();
    let mut linker = wasmtime::Linker::new(&c.data().engine);
    if let Err(e) = super::link::add(&mut linker, cb, eval, prov) {
        return finish(c, prev, guard, auth, delegate, e);
    }
    let result = super::child::run(c, module, &linker, gi, go);
    let ret = match result {
        Ok(0) => 0,
        Ok(n) if n < 0 => n,
        Ok(_) => crate::boot::vm::codes::ERR_INVALID,
        Err(e) => e,
    };
    finish(c, prev, guard, auth, delegate, ret)
}
