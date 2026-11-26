type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap("skrynia", "now", |_c: wasmtime::Caller<'_, Vm>| -> i64 {
            crate::sys::clock::now_millis() as i64
        })
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "secure_random_bits",
            |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, bits: u64| -> i32 {
                let n = match usize::try_from(bits.div_ceil(8)) {
                    Ok(v) => v,
                    Err(_) => return crate::boot::vm::host::ram::args::E_BOUNDS,
                };
                if n > crate::boot::limits::MAX_IO as usize {
                    return crate::boot::vm::host::ram::args::E_BOUNDS;
                }
                let mut b = vec![0u8; n];
                if crate::sys::rand::fill_bits(&mut b, bits as u32).is_err() {
                    return crate::boot::vm::host::ram::args::E_IO;
                }
                match crate::boot::vm::host::ram::args::write(&mut c, ptr, &b) {
                    Ok(()) => 0,
                    Err(e) => e,
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
