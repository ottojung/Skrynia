type Vm = crate::boot::vm::state::Vm;

/// Authority check for one syscall path. The root comes only from the explicit
/// `operator/` or `user/` namespace token; caps can deny but never redirect it.
pub fn resolve(
    c: &wasmtime::Caller<'_, Vm>,
    op: super::super::cap::FsOp,
    path: &[u8],
) -> Result<Option<crate::fs::RootKind>, i32> {
    let (kind, suffix) = crate::fs::root::select::split(path)
        .map_err(|_| crate::boot::vm::host::ram::args::E_INVAL)?;
    if !c.data().fs_auth.allows(kind, op, suffix) {
        return Err(crate::boot::vm::codes::ERR_PERM);
    }
    Ok(Some(kind))
}
