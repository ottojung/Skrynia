pub(super) fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    in_fd: i32,
    out_fd: i32,
) -> i32 {
    let module = match c.data().program1.clone() {
        Some(m) => m,
        None => return crate::boot::vm::codes::ERR_UNSUPPORTED,
    };
    if in_fd == out_fd {
        return crate::boot::vm::codes::ERR_BADFD;
    }
    let input = match c.data_mut().fds.remove(crate::fs::Fd(in_fd)) {
        Some(x) => x,
        None => return crate::boot::vm::codes::ERR_BADFD,
    };
    let output = match c.data_mut().fds.remove(crate::fs::Fd(out_fd)) {
        Some(x) => x,
        None => {
            let _ = c.data_mut().fds.insert(input);
            return crate::boot::vm::codes::ERR_BADFD;
        }
    };
    let engine = c.data().engine.clone();
    let scoped = c.data().program1_storage.clone();
    let fs = scoped.clone().unwrap_or_else(|| c.data().fs.clone());
    let ram = c.data().ram.clone();
    let cache = c.data().cache.clone();
    let ctx = crate::boot::vm::run::program1::Context {
        engine: &engine,
        module: &module,
        fs,
        ram,
        cache,
        scoped: scoped.is_some(),
    };
    match crate::boot::vm::run::program1::run(ctx, input, output) {
        Ok(()) => 0,
        Err(e) => e,
    }
}
