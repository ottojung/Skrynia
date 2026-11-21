pub fn take(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    a: i32,
    b: i32,
) -> Result<(crate::fs::Fd, crate::fs::Fd), i32> {
    if a == b {
        return Err(crate::boot::vm::codes::ERR_BADFD);
    }
    let ia = c
        .data_mut()
        .fds
        .remove(crate::fs::Fd(a))
        .ok_or(crate::boot::vm::codes::ERR_BADFD)?;
    let ib = match c.data_mut().fds.remove(crate::fs::Fd(b)) {
        Some(x) => x,
        None => {
            let _ = c.data_mut().fds.insert(ia, 0);
            return Err(crate::boot::vm::codes::ERR_BADFD);
        }
    };
    let owner = c.data().fd_scope;
    let ga = c
        .data_mut()
        .fds
        .insert(ia, owner)
        .map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let gb = match c.data_mut().fds.insert(ib, owner) {
        Ok(x) => x,
        Err(_) => {
            let _ = c.data_mut().fds.remove(ga);
            return Err(crate::boot::vm::codes::ERR_LIMIT);
        }
    };
    c.data_mut().guest = Some(crate::boot::vm::state::GuestCap { a: ga, b: gb });
    Ok((ga, gb))
}

pub fn cleanup(c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>) {
    if let Some(cap) = c.data_mut().guest.take() {
        let _ = c.data_mut().fds.remove(cap.a);
        let _ = c.data_mut().fds.remove(cap.b);
    }
}
