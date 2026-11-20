pub fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    in_fd: i32,
    out_fd: i32,
    cb: Option<wasmtime::Func>,
) -> i32 {
    let (gi, go) = match super::fds::take(c, in_fd, out_fd) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let mut linker = wasmtime::Linker::new(&c.data().engine);
    if let Err(e) = super::link::add(&mut linker, cb) {
        super::fds::cleanup(c);
        return e;
    }
    let result = (|| {
        let inst = linker
            .instantiate(&mut *c, module)
            .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
        let main: wasmtime::TypedFunc<(i32, i32), i32> = inst
            .get_typed_func(&mut *c, "skrynia_main")
            .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
        let status = main
            .call(&mut *c, (gi.0, go.0))
            .map_err(crate::boot::vm::err::trap)?;
        if status == 0 { Ok(()) } else { Err(status) }
    })();
    super::fds::cleanup(c);
    match result {
        Ok(()) => 0,
        Err(e) => e,
    }
}
