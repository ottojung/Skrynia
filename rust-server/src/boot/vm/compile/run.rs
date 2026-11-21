use std::time::{Duration, Instant};
const MAX_PRE: usize = 64 * 1024 * 1024;
const MAX_OUT: usize = MAX_PRE + 64;
pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    if code.len() > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err("limit");
    }
    let _permit = super::slots::acquire()?;
    let mut child = super::child::spawn()?;
    let writer = super::io::send(child.stdin.take(), code);
    let reader = super::io::receive(child.stdout.take(), MAX_OUT);
    let status = wait_deadline(&mut child);
    let _ = writer.join();
    let out = reader.join().unwrap_or_default();
    match status? {
        0 if out.len() <= MAX_OUT => {
            let payload = super::attest::verify(&out)?;
            unsafe { wasmtime::Module::deserialize(engine, payload).map_err(|_| "invalid") }
        }
        2 => Err("invalid"),
        _ => Err("limit"),
    }
}
fn wait_deadline(child: &mut std::process::Child) -> Result<i32, &'static str> {
    let deadline = Instant::now() + Duration::from_millis(crate::boot::limits::MAX_COMPILE_MS);
    loop {
        match child.try_wait() {
            Ok(Some(st)) => return Ok(st.code().unwrap_or(3)),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(2));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("limit");
            }
        }
    }
}
