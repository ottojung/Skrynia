use std::io::{Read, Write};

const MAX_PRE: usize = 64 * 1024 * 1024;

pub fn main() -> ! {
    std::process::exit(run())
}

fn run() -> i32 {
    let max = crate::boot::vm::state::MAX_MODULE_BYTES;
    let mut buf = Vec::with_capacity(max.min(1024 * 1024));
    if std::io::stdin()
        .take((max + 1) as u64)
        .read_to_end(&mut buf)
        .is_err()
    {
        return 3;
    }
    if buf.len() > max {
        return 3;
    }
    if cfg!(debug_assertions) && buf == b"SKRYNIA_HANG" {
        std::thread::sleep(std::time::Duration::from_secs(5));
        return 3;
    }
    let Some(code) = super::probe::code(&buf) else {
        return 3;
    };
    let cfg = match crate::boot::limits::engine_config() {
        Ok(c) => c,
        Err(_) => return 3,
    };
    let eng = match wasmtime::Engine::new(&cfg) {
        Ok(e) => e,
        Err(_) => return 3,
    };
    match eng.precompile_module(code) {
        Ok(out) if out.len() <= MAX_PRE => {
            if std::io::stdout().write_all(&out).is_err() {
                return 3;
            }
            0
        }
        Ok(_) => 3,
        Err(_) => 2,
    }
}
