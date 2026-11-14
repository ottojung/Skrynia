use super::Fixture;
use std::sync::atomic::Ordering;
#[test]
fn clean_push_io_failure_is_retryable_without_restart() {
    let fixture = Fixture::new(1);
    let pending = fixture.read();
    let file = fixture.path.join("state/push/outbox.json");
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    let report = || {
        vec![(
            fixture.sub.id.clone(),
            pending.clone(),
            Some(fixture.sub.clone()),
            Ok(()),
        )]
    };
    assert!(crate::push_finish_batch(&fixture.state, report()).is_err());
    assert!(!fixture.state.failed.load(Ordering::Acquire));
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
    std::fs::remove_dir(file).unwrap();
    assert!(crate::push_finish_batch(&fixture.state, report()).is_ok());
    assert!(fixture.state.data.lock().unwrap().push.outbox.is_empty());
}
