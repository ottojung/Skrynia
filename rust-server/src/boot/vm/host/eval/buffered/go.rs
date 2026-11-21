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
    let (prev, guard) = match super::super::guest::child::setup(c, true) {
        Ok(x) => x,
        Err(e) => {
            let _ = c.data_mut().fds.remove(in_fd);
            let _ = c.data_mut().fds.remove(out_fd);
            return e;
        }
    };
    let (gi, go) = match super::super::guest::fds::take(c, in_fd.0, out_fd.0) {
        Ok(x) => x,
        Err(e) => {
            let _ = c.data_mut().providers.pop();
            c.data_mut().cur = prev;
            c.data_mut().guest = guard;
            return e;
        }
    };
    let prov = c.data().providers.last().cloned();
    let mut linker = wasmtime::Linker::new(&c.data().engine);
    let ctx = fin::Ctx {
        prev,
        guard,
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
