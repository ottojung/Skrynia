pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "cas_ram_u64",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             fd: i32,
             off: u64,
             exp: u64,
             des: u64,
             op: u32|
             -> i32 { run(&mut c, fd, off, exp, des, op) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn run(
    c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
    fd: i32,
    off: u64,
    exp: u64,
    des: u64,
    op: u32,
) -> i32 {
    if let Err(e) = super::super::fd::guard::check(c.data(), crate::fs::Fd(fd)) {
        return e;
    }
    let ok = matches!(
        c.data().fds.get(crate::fs::Fd(fd)),
        Some(crate::fs::store::Item::Ram(_))
    );
    if !ok {
        return crate::boot::vm::codes::ERR_BADFD;
    }
    let (m, end, off) = match super::check::bounds(c, op, off) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let old = match c.data().ram.cas_u64(off, exp, des) {
        Ok(v) => v,
        Err(e) if e == "oob" || e == "unaligned" => return crate::boot::vm::codes::ERR_RANGE,
        Err(_) => return crate::boot::vm::codes::ERR_FAULT,
    };
    m.data_mut(&mut *c)[op as usize..end].copy_from_slice(&old.to_le_bytes());
    0
}
