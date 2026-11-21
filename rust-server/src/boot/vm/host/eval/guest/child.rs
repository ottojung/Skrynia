pub fn setup(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    eval: bool,
) -> Result<
    (
        Option<wasmtime::Instance>,
        Option<crate::boot::vm::state::GuestCap>,
        crate::boot::vm::cap::FsAuth,
        crate::boot::vm::cap::FsAuth,
    ),
    i32,
> {
    let prev = c.data().cur;
    let guard = c.data().guest;
    let auth = c.data().fs_auth.clone();
    let delegate = c.data().fs_delegate.clone();
    let Some(pinst) = prev else {
        return Err(crate::boot::vm::codes::ERR_FAULT);
    };
    c.data_mut()
        .providers
        .push(crate::boot::vm::state::ProviderCtx {
            inst: pinst,
            guard,
            eval,
            fs_auth: auth.clone(),
            fs_delegate: delegate.clone(),
        });
    c.data_mut().fs_auth = crate::boot::vm::cap::FsAuth::Empty;
    c.data_mut().fs_delegate = crate::boot::vm::cap::FsAuth::Empty;
    Ok((prev, guard, auth, delegate))
}
pub fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    module: &wasmtime::Module,
    linker: &wasmtime::Linker<crate::boot::vm::state::Vm>,
    gi: crate::fs::Fd,
    go: crate::fs::Fd,
) -> Result<i32, i32> {
    let inst = linker
        .instantiate(&mut *c, module)
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    let main: wasmtime::TypedFunc<(i32, i32), i32> =
        inst.get_typed_func(&mut *c, "skrynia_main")
            .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    c.data_mut().cur = Some(inst);
    main.call(&mut *c, (gi.0, go.0))
        .map_err(crate::boot::vm::err::trap)
}
