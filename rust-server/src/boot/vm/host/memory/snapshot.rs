use wasmtime::Caller;

pub fn read(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    len: u64,
) -> Result<Vec<u8>, i32> {
    // Path staging is bounded independently of the size of caller memory.
    if len > crate::boot::limits::MAX_IO as u64 {
        return Err(super::E_BOUNDS);
    }
    let (memory, span) = super::buffer(c, ptr, len)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(span.len())
        .map_err(|_| super::E_IO)?;
    bytes.extend_from_slice(&memory.data(&*c)[span]);
    Ok(bytes)
}
