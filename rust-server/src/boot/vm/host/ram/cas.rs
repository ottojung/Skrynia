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
    let ok = {
        let t = c.data().fs.items.lock().unwrap();
        matches!(
            t.get(crate::fs::Fd(fd)),
            Some(crate::fs::store::Item::Ram(_))
        )
    };
    if !ok {
        return crate::boot::vm::codes::ERR_BADFD;
    }
    let m = match crate::boot::vm::mem::get(c) {
        Ok(x) => x,
        Err(e) => return e,
    };
    if op as usize + 8 > m.data(&c).len() {
        return crate::boot::vm::codes::ERR_RANGE;
    }
    let old = match c.data().ram.cas_u64(off as usize, exp, des) {
        Ok(v) => v,
        Err(e) if e == "oob" || e == "unaligned" => return crate::boot::vm::codes::ERR_RANGE,
        Err(_) => return crate::boot::vm::codes::ERR_FAULT,
    };
    m.data_mut(&mut *c)[op as usize..op as usize + 8].copy_from_slice(&old.to_le_bytes());
    0
}
