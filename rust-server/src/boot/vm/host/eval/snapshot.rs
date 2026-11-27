use wasmtime::Caller;

pub fn snapshot(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, i64> {
    if size > crate::boot::vm::state::MAX_MODULE_BYTES as u64 {
        return Err(-6);
    }
    let (memory, span) =
        crate::boot::vm::host::memory::buffer(c, offset, size).map_err(i64::from)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(span.len()).map_err(|_| -5i64)?;
    bytes.extend_from_slice(&memory.data(&*c)[span]);
    Ok(bytes)
}
