mod codes;
mod range;
mod snapshot;
pub use codes::*;
pub use range::buffer;
pub use snapshot::read;

pub fn write(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    ptr: u64,
    src: &[u8],
) -> Result<(), i32> {
    let (memory, span) = buffer(c, ptr, src.len() as u64)?;
    memory.data_mut(c)[span].copy_from_slice(src);
    Ok(())
}
