// Transitional old-ABI host import; removed when K2 ports eval tests.
type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "cas_ram_u64",
            |mut c: wasmtime::Caller<'_, Vm>,
             _fd: i32,
             off: u64,
             exp: u64,
             des: u64,
             op: u32|
             -> i32 {
                let (m, end, off) = match check(&mut c, off, op) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let old = match c.data().ram.cas_u64(off, exp, des) {
                    Ok(v) => v,
                    Err(_) => return crate::boot::vm::codes::ERR_RANGE,
                };
                m.data_mut(&mut c)[op as usize..end].copy_from_slice(&old.to_le_bytes());
                0
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn check(
    c: &mut wasmtime::Caller<'_, Vm>,
    off: u64,
    ptr: u32,
) -> Result<(wasmtime::Memory, usize, usize), i32> {
    let m = crate::boot::vm::mem::get(c)?;
    let end = (ptr as usize)
        .checked_add(8)
        .ok_or(crate::boot::vm::codes::ERR_RANGE)?;
    if end > m.data(&*c).len() {
        return Err(crate::boot::vm::codes::ERR_RANGE);
    }
    let offset = usize::try_from(off).map_err(|_| crate::boot::vm::codes::ERR_RANGE)?;
    Ok((m, end, offset))
}
