use libskrynia::boot::vm::state::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn failure_not_cached_and_retries() {
    let eng = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&eng, &wat::parse_str("(module)").unwrap()).unwrap();
    let cache = Cache::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let n1 = calls.clone();
    let r1 = cache.compile(b"same", || {
        n1.fetch_add(1, Ordering::SeqCst);
        Err("invalid")
    });
    let n2 = calls.clone();
    let m = module.clone();
    let r2 = cache.compile(b"same", || {
        n2.fetch_add(1, Ordering::SeqCst);
        Ok(m)
    });
    assert_eq!(r1.err(), Some("invalid"));
    assert!(r2.is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(cache.len(), 1);
}
