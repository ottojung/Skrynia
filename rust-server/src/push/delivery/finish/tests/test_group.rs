use super::Fixture;
use crate::{PushFailure, push_finish_batch};
#[test]
fn successful_ack_removes_snapshot_only_notification_durably() {
    let f = Fixture::new(1);
    let pending = f.read();
    push_finish_batch(
        &f.state,
        vec![(f.sub.id.clone(), pending, Some(f.sub.clone()), Ok(()))],
    )
    .unwrap();
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
    assert!(crate::read_outbox(&f.path).unwrap().is_empty());
}

#[test]
fn blocked_dns_address_is_retried_without_retiring_subscription() {
    let f = Fixture::new(1);
    let original = f.read();
    push_finish_batch(
        &f.state,
        vec![(
            f.sub.id.clone(),
            original,
            Some(f.sub.clone()),
            Err(PushFailure::Retry),
        )],
    )
    .unwrap();
    let next = f.read();
    assert_eq!(next.attempts, 1);
    assert!(next.next_at >= crate::push_now());
    assert!(
        f.path
            .join("state/push-subs")
            .join(format!("{}.json", f.sub.id))
            .exists()
    );
    assert!(
        f.state
            .data
            .lock()
            .unwrap()
            .push
            .subscriptions
            .contains_key(&f.sub.id)
    );
}
