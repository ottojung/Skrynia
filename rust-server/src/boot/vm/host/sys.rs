type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "now",
            |_c: wasmtime::Caller<'_, Vm>| -> i64 { crate::sys::clock::now_millis() as i64 },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia_v3_1",
            "secure_random_bits",
            |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, bits: u64| -> i32 {
                let n = match usize::try_from(bits.div_ceil(8)) {
                    Ok(v) => v,
                    Err(_) => return crate::boot::vm::host::memory::E_BOUNDS,
                };
                if n > crate::boot::limits::MAX_IO as usize {
                    return crate::boot::vm::host::memory::E_BOUNDS;
                }
                if let Err(e) = crate::boot::vm::host::memory::buffer(&mut c, ptr, n as u64) {
                    return e;
                }
                let mut b = Vec::new();
                if b.try_reserve_exact(n).is_err() {
                    return crate::boot::vm::host::memory::E_IO;
                }
                b.resize(n, 0);
                if crate::sys::rand::fill_bits(&mut b, bits as u32).is_err() {
                    return crate::boot::vm::host::memory::E_IO;
                }
                match crate::boot::vm::host::memory::write(&mut c, ptr, &b) {
                    Ok(()) => 0,
                    Err(e) => e,
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
