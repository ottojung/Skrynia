type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "next_entry",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, dst: u64, cap: u64| -> i32 {
                if crate::boot::vm::host::fd::guard::check(c.data(), crate::fs::Fd(fd)).is_err() {
                    return crate::boot::vm::host::ram::args::E_BADFD;
                }
                let max = cap.min(4096);
                if max > crate::boot::limits::MAX_IO as u64 {
                    return crate::boot::vm::host::ram::args::E_BOUNDS;
                }
                let mut buf = vec![0u8; max as usize];
                let fs = c.data().fs.clone();
                let res = fs.next_entry(&mut c.data_mut().fds, crate::fs::Fd(fd), &mut buf);
                let out = match res {
                    Ok(crate::fs::dir::Next::Name(n)) => n,
                    Ok(crate::fs::dir::Next::End) => return 0,
                    Ok(crate::fs::dir::Next::Small) => {
                        return crate::boot::vm::host::ram::args::E_BOUNDS;
                    }
                    Err(e) => return crate::boot::vm::host::ram::args::map_fs(e),
                };
                match crate::boot::vm::host::ram::args::write(&mut c, dst, &buf[..out]) {
                    Ok(()) => out as i32,
                    Err(e) => e,
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
