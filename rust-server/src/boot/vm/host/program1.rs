pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "run_program1",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             in_fd: i32,
             out_fd: i32|
             -> i32 { run(&mut c, in_fd, out_fd) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn run(c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>, in_fd: i32, out_fd: i32) -> i32 {
    let module = match c.data().program1.clone() {
        Some(m) => m,
        None => return crate::boot::vm::codes::ERR_UNSUPPORTED,
    };
    if in_fd == out_fd {
        return crate::boot::vm::codes::ERR_BADFD;
    }
    let input = match c.data_mut().fds.remove(crate::fs::Fd(in_fd)) {
        Some(x) => x,
        None => return crate::boot::vm::codes::ERR_BADFD,
    };
    let output = match c.data_mut().fds.remove(crate::fs::Fd(out_fd)) {
        Some(x) => x,
        None => {
            let _ = c.data_mut().fds.insert(input);
            return crate::boot::vm::codes::ERR_BADFD;
        }
    };
    let engine = c.data().engine.clone();
    let fs = c.data().fs.clone();
    let ram = c.data().ram.clone();
    let cache = c.data().cache.clone();
    match crate::boot::vm::run::program1::run(&engine, &module, fs, ram, cache, input, output) {
        Ok(()) => 0,
        Err(e) => e,
    }
}
