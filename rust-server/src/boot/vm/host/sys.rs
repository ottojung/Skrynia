pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "now",
            |_c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i64 {
                crate::sys::clock::now_millis() as i64
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "secure_random_bits",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, ptr: u32, bits: u32| -> i32 {
                let n = (bits as usize).div_ceil(8);
                if n > crate::boot::limits::MAX_IO as usize {
                    return crate::boot::vm::codes::ERR_LIMIT;
                }
                let mut b = vec![0u8; n];
                if crate::sys::rand::fill_bits(&mut b, bits).is_err() {
                    return crate::boot::vm::codes::ERR_IO;
                }
                let m = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                if crate::boot::vm::mem::write_buf(&mut c, m, ptr, &b).is_err() {
                    return crate::boot::vm::codes::ERR_RANGE;
                }
                0
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "eval_output_len",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                c.data()
                    .frames
                    .first()
                    .map(|f| f.last_overflow)
                    .unwrap_or(-1)
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
