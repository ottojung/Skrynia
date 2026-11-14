use super::Fixture;
use crate::{PushFailure, push_finish_batch};
use std::{fs, sync::atomic::Ordering};
#[test]
fn transient_outbox_failure_retains_delivery_failure_count_until_recovery() {
    let f = Fixture::new(1);
    let initial = f.read();
    let path = f.path.join("state/push/outbox.json");
    let saved = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let retry = || {
        vec![(
            f.sub.id.clone(),
            initial.clone(),
            Some(f.sub.clone()),
            Err(PushFailure::Retry),
        )]
    };
    for attempts in 1..=3 {
        assert!(push_finish_batch(&f.state, retry()).is_err());
        let data = f.state.data.lock().unwrap();
        let old = &data.push.outbox[&f.sub.id];
        assert_eq!(old.attempts, 0, "disk state must remain unchanged");
        assert_eq!(data.push.retry_attempts(&f.sub.id, old), attempts);
        assert!(!data.push.ready(&f.sub.id, old, crate::push_now()));
        assert!(!f.state.failed.load(Ordering::Acquire));
    }
    fs::remove_dir(&path).unwrap();
    fs::write(&path, saved).unwrap();
    push_finish_batch(&f.state, retry()).unwrap();
    assert!(f.state.data.lock().unwrap().push.subscriptions.is_empty());
    assert!(crate::read_outbox(&f.path).unwrap().is_empty());
}
