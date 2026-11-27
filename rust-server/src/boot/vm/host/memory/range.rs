use std::ops::Range;
use wasmtime::{Caller, Memory};

pub fn buffer(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    len: u64,
) -> Result<(Memory, Range<usize>), i32> {
    // get_export selects only an ordinary, non-shared linear memory.
    let memory = c
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or(super::E_INVAL)?;
    let end = ptr.checked_add(len).ok_or(super::E_RANGE)?;
    let start = usize::try_from(ptr).map_err(|_| super::E_BOUNDS)?;
    let end = usize::try_from(end).map_err(|_| super::E_BOUNDS)?;
    if end > memory.data_size(&*c) || len > i64::MAX as u64 {
        return Err(super::E_BOUNDS);
    }
    Ok((memory, start..end))
}
