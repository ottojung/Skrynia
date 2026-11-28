type Vm = crate::boot::vm::state::Vm;
type Caller<'a> = wasmtime::Caller<'a, Vm>;

fn transfer(mut c: Caller<'_>, fd: i32, ptr: u64, len: u64, write: bool) -> i64 {
    let (memory, span) = match crate::boot::vm::host::memory::buffer(&mut c, ptr, len) {
        Ok(v) => v,
        Err(e) => return i64::from(e),
    };
    // No memory view survives this host call or a nested WASM invocation.
    let (bytes, vm) = memory.data_and_store_mut(&mut c);
    let fs = vm.fs.clone();
    let fd = crate::fs::Fd(fd);
    let result = if write {
        fs.write(&mut vm.fds, fd, &bytes[span])
    } else {
        fs.read(&mut vm.fds, fd, &mut bytes[span])
    };
    match result {
        Ok(n) => n as i64,
        Err(e) => i64::from(crate::boot::vm::host::memory::map_fs(e)),
    }
}

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "read",
            |c: Caller<'_>, fd: i32, ptr: u64, len: u64| -> i64 {
                transfer(c, fd, ptr, len, false)
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia_v3_1",
            "write",
            |c: Caller<'_>, fd: i32, ptr: u64, len: u64| -> i64 { transfer(c, fd, ptr, len, true) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
