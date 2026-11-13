use super::Fixture;
use crate::{PushFailure, push_finish_batch};
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
    push_finish_batch(
        &f.state,
        vec![(
            sent.id.clone(),
            current,
            Some(sent.clone()),
            Err(PushFailure::Gone),
        )],
    )
    .unwrap();
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
