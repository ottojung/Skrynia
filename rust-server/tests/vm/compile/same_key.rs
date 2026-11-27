use libskrynia::boot::vm::state::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

#[test]
fn same_key_compiles_once() {
    let eng = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&eng, &wat::parse_str("(module)").unwrap()).unwrap();
    let cache = Arc::new(Cache::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel::<()>();
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let go_rx = Arc::new(std::sync::Mutex::new(go_rx));
    let mut hs = Vec::new();
    for _ in 0..8 {
        let c = cache.clone();
        let n = calls.clone();
        let tx = tx.clone();
        let go_rx = go_rx.clone();
        let m = module.clone();
        hs.push(std::thread::spawn(move || {
            c.compile(b"same", || {
                n.fetch_add(1, Ordering::SeqCst);
                tx.send(()).unwrap();
                go_rx.lock().unwrap().recv().unwrap();
                Ok(m)
            })
        }));
    }
    rx.recv().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    go_tx.send(()).unwrap();
    for h in hs {
        assert!(h.join().unwrap().is_ok());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
