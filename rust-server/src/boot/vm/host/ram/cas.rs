type Vm = crate::boot::vm::state::Vm;
type Caller<'a> = wasmtime::Caller<'a, Vm>;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "cas1",
            |mut c: Caller<'_>, fd: i32, off: u64, exp: u32, des: u32| -> i32 {
                run(&mut c, fd, off, exp, des)
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn run(c: &mut Caller<'_>, fd: i32, off: u64, exp: u32, des: u32) -> i32 {
    if exp > 255 || des > 255 {
        return crate::boot::vm::host::ram::args::E_INVAL;
    }
    if crate::boot::vm::host::fd::guard::check(c.data(), crate::fs::Fd(fd)).is_err() {
        return crate::boot::vm::host::ram::args::E_BADFD;
    }
    let item = c.data().fds.get(crate::fs::Fd(fd));
    if item.is_none() {
        return crate::boot::vm::host::ram::args::E_BADFD;
    }
    if !matches!(item, Some(crate::fs::store::Item::Ram(_))) {
        return crate::boot::vm::host::ram::args::E_NOSUP;
    }
    let ram = c.data().ram.clone();
    match ram.cas1(off, exp as u8, des as u8) {
        Ok(v) => i32::from(v),
        Err("oob") => crate::boot::vm::host::ram::args::E_BOUNDS,
        Err(_) => crate::boot::vm::host::ram::args::E_IO,
    }
}
