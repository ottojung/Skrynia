use std::io::{Read, Write};
use std::time::{Duration, Instant};

const MAX_PRE: usize = 64 * 1024 * 1024;

pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    if code.len() > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err("limit");
    }
    let mut child = super::child::spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(code);
    }
    let out_h = read_pipe(child.stdout.take());
    let _ = read_pipe(child.stderr.take());
    let status = wait_deadline(&mut child)?;
    let out = out_h.join().unwrap_or_default();
    match status {
        0 if out.len() <= MAX_PRE => unsafe {
            wasmtime::Module::deserialize(engine, &out).map_err(|_| "invalid")
        },
        2 => Err("invalid"),
        _ => Err("limit"),
    }
}

fn read_pipe<T: Read + Send + 'static>(p: Option<T>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut v = Vec::new();
        if let Some(r) = p {
            let _ = r.take((MAX_PRE + 1) as u64).read_to_end(&mut v);
        }
        v
    })
}

fn wait_deadline(child: &mut std::process::Child) -> Result<i32, &'static str> {
    let ms = crate::boot::limits::MAX_COMPILE_MS;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) => return Ok(st.code().unwrap_or(3)),
            Ok(None) if start.elapsed() < Duration::from_millis(ms) => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("limit");
            }
            Err(_) => return Err("limit"),
        }
    }
}
