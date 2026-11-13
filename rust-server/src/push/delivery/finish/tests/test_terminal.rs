use super::Fixture;
use crate::{PushFailure, push_finish};
#[test]
fn terminal_failure_retires_only_unchanged_subscription() {
    let f = Fixture::new(1);
    let sent = f.sub.clone();
    let current = f.read();
    f.state
        .data
        .lock()
        .unwrap()
        .push
        .subscriptions
        .get_mut(&sent.id)
        .unwrap()
        .cap_hash = "f".repeat(64);
    push_finish(&f.state, &current, Some(&sent), Err(PushFailure::Gone)).unwrap();
    assert!(
        f.state
            .data
            .lock()
            .unwrap()
            .push
            .subscriptions
            .contains_key(&sent.id)
    );
    assert_eq!(f.read().attempts, 0);
    assert_eq!(f.read().next_at, 0);
}
#[test]
fn gone_endpoint_is_durably_retired() {
    let f = Fixture::new(1);
    let current = f.read();
    push_finish(&f.state, &current, Some(&f.sub), Err(PushFailure::Gone)).unwrap();
    assert!(
        !f.path
            .join("state/push-subs")
            .join(format!("{}.json", f.sub.id))
            .exists()
    );
    assert!(f.state.data.lock().unwrap().push.subscriptions.is_empty());
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
}
