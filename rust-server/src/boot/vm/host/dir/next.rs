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
                let res = c.data().fs.next_entry(crate::fs::Fd(fd), &mut buf);
                match res {
                    Ok(crate::fs::dir::Next::Name(n)) => {
                        let m = match crate::boot::vm::mem::get(&mut c) {
                            Ok(x) => x,
                            Err(e) => return e,
                        };
                        if crate::boot::vm::mem::write_buf(&mut c, m, dst, &buf[..n]).is_err() {
                            return crate::boot::vm::codes::ERR_RANGE;
                        }
                        n as i32
                    }
                    Ok(crate::fs::dir::Next::End) => 0,
                    Ok(crate::fs::dir::Next::Small) => crate::boot::vm::codes::ERR_OVERFLOW,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
