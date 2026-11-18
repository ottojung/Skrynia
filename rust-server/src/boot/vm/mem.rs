use super::codes::{ERR_FAULT, ERR_RANGE};
use wasmtime::{Caller, Instance, Memory, Store};

pub fn get<T>(caller: &mut Caller<'_, T>) -> Result<Memory, i32> {
    caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or(ERR_FAULT)
}
pub fn read_buf<T>(
    caller: &mut Caller<'_, T>,
    mem: Memory,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, i32> {
    if len > crate::boot::limits::MAX_IO {
        return Err(super::codes::ERR_LIMIT);
    }
    let mut buf = vec![0u8; len as usize];
    mem.read(caller, ptr as usize, &mut buf)
        .map_err(|_| ERR_RANGE)?;
    Ok(buf)
}
pub fn write_buf<T>(
    caller: &mut Caller<'_, T>,
    mem: Memory,
    ptr: u32,
    src: &[u8],
) -> Result<(), i32> {
    mem.write(caller, ptr as usize, src).map_err(|_| ERR_RANGE)
}
pub fn get_in<T>(store: &mut Store<T>, inst: Instance) -> Result<Memory, i32> {
    inst.get_memory(&mut *store, "memory").ok_or(ERR_FAULT)
}
pub fn read_inst<T>(store: &mut Store<T>, mem: Memory, ptr: u32, len: u32) -> Result<Vec<u8>, i32> {
    if len > crate::boot::limits::MAX_IO {
        return Err(super::codes::ERR_LIMIT);
    }
    let mut buf = vec![0u8; len as usize];
    mem.read(&mut *store, ptr as usize, &mut buf)
        .map_err(|_| ERR_RANGE)?;
    Ok(buf)
}
pub fn write_inst<T>(store: &mut Store<T>, mem: Memory, ptr: u32, src: &[u8]) -> Result<(), i32> {
    mem.write(&mut *store, ptr as usize, src)
        .map_err(|_| ERR_RANGE)
}
