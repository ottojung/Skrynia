use wasmtime::Caller;

const ERR_RANGE: i64 = -6;

pub fn snapshot(
    c: &Caller<'_, crate::boot::vm::state::Vm>,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, i64> {
    if size > crate::boot::vm::state::MAX_MODULE_BYTES as u64 {
        return Err(ERR_RANGE);
    }
    let offset = usize::try_from(offset).map_err(|_| ERR_RANGE)?;
    let size = usize::try_from(size).map_err(|_| ERR_RANGE)?;
    let mut bytes = vec![0u8; size];
    let n = c
        .data()
        .ram
        .read(offset, &mut bytes)
        .map_err(|_| ERR_RANGE)?;
    if n != size {
        return Err(ERR_RANGE);
    }
    Ok(bytes)
}
