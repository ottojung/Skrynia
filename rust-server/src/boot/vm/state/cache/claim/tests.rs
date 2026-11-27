use super::{Cache, Claim};

#[test]
fn delayed_claim_reuses_completed_module_after_optimistic_miss() {
    let cache = Cache::new();
    let code = wat::parse_str("(module)").unwrap();
    let key = Cache::key(&code);
    assert!(cache.get(&code).is_none());
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &code).unwrap();
    cache.compile(&code, || Ok(module)).unwrap();
    assert!(cache.inflight.lock().unwrap().is_empty());
    // Resume a caller which observed the miss before the prior compile.
    assert!(matches!(cache.claim(key), Claim::Ready(_)));
    assert!(cache.inflight.lock().unwrap().is_empty());
}
