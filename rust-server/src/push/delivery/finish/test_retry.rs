use super::fixture::Fixture;
use crate::{Notification, PushFailure, push_finish};
#[test]
fn failed_send_retains_latest_generation_and_backoff() {
    let f = Fixture::new(2);
    let stale = Notification {
        subscription_id: f.sub.id.clone(),
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push_finish(&f.state, &stale, Some(&f.sub), Err(PushFailure::Retry)).unwrap();
    let pending = f.read();
    assert_eq!(pending.generation, 2);
    assert_eq!(pending.attempts, 1);
    assert!(pending.next_at >= crate::push_now());
}

#[test]
fn blocked_dns_address_is_retried_without_retiring_subscription() {
    let fixture = Fixture::new(1);
    let notification = fixture.read();
    push_finish(
        &fixture.state,
        &notification,
        Some(&fixture.sub),
        Err(PushFailure::Retry),
    )
    .unwrap();
    let next = fixture.read();
    assert_eq!(next.attempts, 1);
    assert!(next.next_at >= crate::push_now());
    assert!(
        fixture
            .path
            .join("state/push-subs")
            .join(format!("{}.json", fixture.sub.id))
            .exists()
    );
    assert!(
        fixture
            .state
            .data
            .lock()
            .unwrap()
            .push
            .subscriptions
            .contains_key(&fixture.sub.id)
    );
}
