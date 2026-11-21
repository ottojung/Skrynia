use super::{fin, prep::Args};
pub fn exec(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    a: &Args,
    in_fd: crate::fs::Fd,
    out_fd: crate::fs::Fd,
    out: crate::fs::store::item::BufferedOutput,
    cb: Option<wasmtime::Func>,
    module: &wasmtime::Module,
) -> i32 {
    let (prev, guard, auth, delegate) = match super::super::guest::child::setup(c, true) {
        Ok(x) => x,
        Err(e) => {
            let _ = c.data_mut().fds.remove(in_fd);
            let _ = c.data_mut().fds.remove(out_fd);
            return e;
        }
    };
    // Buffered child is an untrusted leaf: no ambient fs authority, no delegated authority.
    c.data_mut().fs_auth = crate::boot::vm::cap::FsAuth::Empty;
    c.data_mut().fs_delegate = crate::boot::vm::cap::FsAuth::Empty;
    let (gi, go) = match super::super::guest::fds::take(c, in_fd.0, out_fd.0) {
        Ok(x) => x,
        Err(e) => {
            let _ = c.data_mut().providers.pop();
            c.data_mut().cur = prev;
            c.data_mut().guest = guard;
            c.data_mut().fs_auth = auth;
            c.data_mut().fs_delegate = delegate;
            return e;
        }
    };
    let prov = c.data().providers.last().cloned();
    let mut linker = wasmtime::Linker::new(&c.data().engine);
    let ctx = fin::Ctx {
        prev,
        guard,
        auth,
        delegate,
        go,
        out,
        res_ptr: a.res_ptr,
        res_cap: a.res_cap,
    };
    if let Err(e) = super::super::guest::link::add(&mut linker, cb, false, prov) {
        return fin::done(c, ctx, Err(e));
    }
    let result = super::super::guest::child::run(c, module, &linker, gi, go);
    fin::done(c, ctx, result)
}
