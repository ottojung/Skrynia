use libskrynia::boot::vm::state::Cache;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

#[test]
fn panic_wakes_waiters() {
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
                go_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .unwrap();
                panic!("boom");
            })
        }));
    }
    rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    go_tx.send(()).unwrap();
    drop(go_tx);
    let mut panic_count = 0;
    let mut err_count = 0;
    for h in hs {
        match h.join() {
            Err(_) => panic_count += 1,
            Ok(Err("limit")) => err_count += 1,
            Ok(other) => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(panic_count, 1);
    assert_eq!(err_count, 5);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
