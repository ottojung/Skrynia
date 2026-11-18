pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "next_entry",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             fd: i32,
             dst: u32,
             cap: u32|
             -> i32 {
                let mut buf = vec![0u8; cap as usize];
                let fs = c.data().fs.clone();
                let res = fs.next_entry(&mut c.data_mut().fds, crate::fs::Fd(fd), &mut buf);
                let out = match res {
                    Ok(crate::fs::dir::Next::Name(n)) => n,
                    Ok(crate::fs::dir::Next::End) => return 0,
                    Ok(crate::fs::dir::Next::Small) => return crate::boot::vm::codes::ERR_OVERFLOW,
                    Err(e) => return crate::boot::vm::err::map(e),
                };
                let m = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                if crate::boot::vm::mem::write_buf(&mut c, m, dst, &buf[..out]).is_err() {
                    return crate::boot::vm::codes::ERR_RANGE;
                }
                out as i32
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
