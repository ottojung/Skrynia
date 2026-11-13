use super::fixture::Fixture;
use std::sync::{atomic::Ordering, mpsc};
#[test]
fn queued_mutation_rechecks_health_after_acquiring_lock() {
    let f = Fixture::new(1);
    let lock = f.state.data.lock().unwrap();
    let (started, ready) = mpsc::channel();
    std::thread::scope(|scope| {
        let write = scope.spawn(|| {
            started.send(()).unwrap();
            f.state.change("queued", "create", b"bytes", None, "bad")
        });
        ready.recv().unwrap();
        f.state.failed.store(true, Ordering::Release);
        drop(lock);
        assert_eq!(
            write.join().unwrap().unwrap_err().0,
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    });
    assert!(!f.path.join("store/queued").exists());
}
