use super::Fixture;
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

#[test]
fn push_io_failure_blocks_later_delivery_completions() {
    let fixture = super::Fixture::new(1);
    let pending = fixture.read();
    let file = fixture
        .path
        .join("state/push/outbox")
        .join(format!("{}.json", fixture.sub.id));
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    assert!(crate::push_finish(&fixture.state, &pending, Some(&fixture.sub), Ok(())).is_err());
    assert!(fixture.state.failed.load(Ordering::Acquire));
    assert!(
        fixture
            .state
            .data
            .lock()
            .unwrap()
            .push
            .outbox
            .contains_key(&fixture.sub.id)
    );
    assert!(crate::push_finish(&fixture.state, &pending, Some(&fixture.sub), Ok(())).is_err());
}
