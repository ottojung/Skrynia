/// Resolve only the explicit operator/user namespace token.
pub fn resolve(path: &[u8]) -> Result<Option<crate::fs::RootKind>, i32> {
    let (kind, _) = crate::fs::root::select::split(path)
        .map_err(|_| crate::boot::vm::host::ram::args::E_INVAL)?;
    Ok(Some(kind))
}
