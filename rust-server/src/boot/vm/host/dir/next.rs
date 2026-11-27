type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "next_entry",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, dst: u64, cap: u64| -> i32 {
                if let Err(e) = crate::boot::vm::host::memory::buffer(&mut c, dst, cap) {
                    return e;
                }
                let max = cap.min(4096);
                if max > crate::boot::limits::MAX_IO as u64 {
                    return crate::boot::vm::host::memory::E_BOUNDS;
                }
                let mut buf = vec![0u8; max as usize];
                let fs = c.data().fs.clone();
                let res = fs.next_entry(&mut c.data_mut().fds, crate::fs::Fd(fd), &mut buf);
                let out = match res {
                    Ok(crate::fs::dir::Next::Name(n)) => n,
                    Ok(crate::fs::dir::Next::End) => return 0,
                    Ok(crate::fs::dir::Next::Small) => {
                        return crate::boot::vm::host::memory::E_BOUNDS;
                    }
                    Err(e) => return crate::boot::vm::host::memory::map_fs(e),
                };
                match crate::boot::vm::host::memory::write(&mut c, dst, &buf[..out]) {
                    Ok(()) => out as i32,
                    Err(e) => e,
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
