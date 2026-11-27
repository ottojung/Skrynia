use libskrynia::boot::vm::state::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

#[test]
fn failure_shared_with_waiters() {
    let cache = Arc::new(Cache::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel::<()>();
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let go_rx = Arc::new(std::sync::Mutex::new(go_rx));
    let mut hs = Vec::new();
    for _ in 0..6 {
        let (c, n, tx, go_rx) = (cache.clone(), calls.clone(), tx.clone(), go_rx.clone());
        hs.push(std::thread::spawn(move || {
            c.compile(b"k", || {
                n.fetch_add(1, Ordering::SeqCst);
                tx.send(()).unwrap();
                go_rx.lock().unwrap().recv().unwrap();
                Err("invalid")
            })
        }));
    }
    rx.recv().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    go_tx.send(()).unwrap();
    for h in hs {
        assert_eq!(h.join().unwrap().err(), Some("invalid"));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
