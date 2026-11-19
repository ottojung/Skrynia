use std::sync::mpsc;
use std::time::Duration;
pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    let (tx, rx) = mpsc::channel();
    let eng = engine.clone();
    let bytes = code.to_vec();
    std::thread::spawn(move || {
        let _ = tx.send(wasmtime::Module::new(&eng, &bytes));
    });
    match rx.recv_timeout(Duration::from_millis(crate::boot::limits::MAX_COMPILE_MS)) {
        Ok(Ok(m)) => Ok(m),
        Ok(Err(_)) => Err("invalid"),
        Err(_) => Err("limit"),
    }
}
