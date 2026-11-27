use libskrynia::boot::vm::state::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

#[test]
fn distinct_keys_compile_independently() {
    let eng = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&eng, &wat::parse_str("(module)").unwrap()).unwrap();
    let cache = Arc::new(Cache::new());
    let ca = Arc::new(AtomicUsize::new(0));
    let cb = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel::<()>();
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let c = cache.clone();
    let n = ca.clone();
    let m = module.clone();
    let ha = std::thread::spawn(move || {
        c.compile(b"a", || {
            n.fetch_add(1, Ordering::SeqCst);
            tx.send(()).unwrap();
            go_rx.recv().unwrap();
            Ok(m)
        })
    });
    rx.recv().unwrap();
    let m2 = module.clone();
    let n2 = cb.clone();
    let rb = cache.compile(b"b", || {
        n2.fetch_add(1, Ordering::SeqCst);
        Ok(m2)
    });
    go_tx.send(()).unwrap();
    assert!(ha.join().unwrap().is_ok());
    assert!(rb.is_ok());
    assert_eq!(ca.load(Ordering::SeqCst), 1);
    assert_eq!(cb.load(Ordering::SeqCst), 1);
}
