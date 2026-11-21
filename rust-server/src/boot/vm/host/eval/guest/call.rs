pub fn invoke(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    cb: wasmtime::Func,
    prov: crate::boot::vm::state::ProviderCtx,
    req: &[u8],
    res_ptr: u32,
    res_cap: u32,
) -> i32 {
    if res_cap > crate::boot::limits::MAX_IO {
        return crate::boot::vm::codes::ERR_LIMIT;
    }
    let Some(p_mem) = prov.inst.get_memory(&mut *c, "memory") else {
        return crate::boot::vm::codes::ERR_FAULT;
    };
    let (base, res_base) = match super::stage::buf(c, p_mem, req, res_cap) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let Ok(cb_t) = cb.typed::<(u32, u32, u32, u32), i32>(&*c) else {
        return crate::boot::vm::codes::ERR_INVALID;
    };
    let saved_guard = c.data_mut().guest.take();
    let saved_cur = c.data().cur;
    let saved_auth = c.data().fs_auth.clone();
    let saved_delegate = c.data().fs_delegate.clone();
    let saved_scope = c.data().fd_scope;
    let scope = prov.callback_scope;
    c.data_mut().guest = prov.guard;
    c.data_mut().cur = Some(prov.inst);
    c.data_mut().fs_auth = prov.fs_delegate.clone();
    c.data_mut().fs_delegate = crate::boot::vm::cap::FsAuth::Empty;
    c.data_mut().fd_scope = scope;
    let call = cb_t.call(&mut *c, (base, req.len() as u32, res_base, res_cap));
    c.data_mut().cur = saved_cur;
    c.data_mut().guest = saved_guard;
    c.data_mut().fs_auth = saved_auth;
    c.data_mut().fs_delegate = saved_delegate;
    c.data_mut().fd_scope = saved_scope;
    match call {
        Ok(n) if n < 0 => n,
        Ok(n) => super::copy::res(c, p_mem, res_base, res_ptr, n as u32, res_cap),
        Err(e) => crate::boot::vm::err::trap(e),
    }
}
