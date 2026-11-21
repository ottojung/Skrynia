pub fn resolve(
    c: &wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    op: super::super::cap::FsOp,
    path: &[u8],
) -> Result<Option<crate::fs::RootKind>, i32> {
    c.data()
        .fs_auth
        .resolve(op, path)
        .ok_or(crate::boot::vm::codes::ERR_PERM)
}
