#[rustfmt::skip]
fn readable(it: &crate::fs::store::Item) -> bool {
    matches!(it, crate::fs::store::Item::Input(_) | crate::fs::store::Item::Ram(_))
        || matches!(it, crate::fs::store::Item::File(e) if e.can_read)
}
#[rustfmt::skip]
fn writable(it: &crate::fs::store::Item) -> bool {
    matches!(it, crate::fs::store::Item::Output(_) | crate::fs::store::Item::BufferedOutput(_) | crate::fs::store::Item::Ram(_))
        || matches!(it, crate::fs::store::Item::File(e) if e.can_write)
}
#[rustfmt::skip]
pub fn take(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>, a: i32, b: i32,
) -> Result<(crate::fs::Fd, crate::fs::Fd), i32> {
    if a == b { return Err(crate::boot::vm::codes::ERR_BADFD); }
    let (fa, fb) = (crate::fs::Fd(a), crate::fs::Fd(b));
    for fd in [fa, fb] { super::super::super::fd::guard::scope(c.data(), fd)?; }
    let ia = c.data().fds.get(fa).ok_or(crate::boot::vm::codes::ERR_BADFD)?;
    let ib = c.data().fds.get(fb).ok_or(crate::boot::vm::codes::ERR_BADFD)?;
    if !readable(ia) || !writable(ib) { return Err(crate::boot::vm::codes::ERR_BADFD); }
    let ia = c.data_mut().fds.remove(fa).ok_or(crate::boot::vm::codes::ERR_BADFD)?;
    let ib = c.data_mut().fds.remove(fb).ok_or(crate::boot::vm::codes::ERR_BADFD)?;
    let owner = c.data().fd_scope;
    let ga = c.data_mut().fds.insert(ia, owner).map_err(|_| crate::boot::vm::codes::ERR_LIMIT)?;
    let gb = match c.data_mut().fds.insert(ib, owner) {
        Ok(x) => x,
        Err(_) => { let _ = c.data_mut().fds.remove(ga); return Err(crate::boot::vm::codes::ERR_LIMIT); }
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
