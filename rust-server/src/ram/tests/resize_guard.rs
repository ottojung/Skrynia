use super::Ram;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn truncate_waits_for_span_guard_release() {
    let r = Arc::new(Ram::new());
    r.truncate(8192).unwrap();
    r.write_byte(4096, 7).unwrap();
    let (tx, rx) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let span_ram = r.clone();
    let check_ram = r.clone();
    let h = thread::spawn(move || {
        let _ = span_ram.span(4096, 1, false, |_page, _at, _n| {
            assert!(check_ram.pages.shard(1).try_write().is_err());
            let _ = tx.send(());
            let _ = rx2.recv();
        });
    });
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let trunc_ram = r.clone();
    let th = thread::spawn(move || trunc_ram.truncate(0).unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    while r.len() != 0 && Instant::now() < deadline {
        thread::yield_now();
    }
    let published = r.len() == 0;
    let finished = th.is_finished();
    let _ = tx2.send(());
    h.join().unwrap();
    th.join().unwrap();
    assert!(published, "truncate did not publish size");
    assert!(!finished, "truncate finished while span held guard");
    assert_eq!(r.len(), 0);
}
