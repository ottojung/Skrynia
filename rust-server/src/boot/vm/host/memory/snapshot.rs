use wasmtime::Caller;

pub fn read(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    len: u64,
) -> Result<Vec<u8>, i32> {
    let (memory, span) = super::buffer(c, ptr, len)?;
    // This snapshot is used only for paths: reject oversized paths before
    // allocating, using the longest root token plus the relative-path bound.
    const MAX_PATH: usize = crate::safe::MAX_RELATIVE_PATH_BYTES + b"operator/".len();
    if span.len() > MAX_PATH {
        return Err(super::E_INVAL);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(span.len())
        .map_err(|_| super::E_IO)?;
    bytes.extend_from_slice(&memory.data(&*c)[span]);
    Ok(bytes)
}
