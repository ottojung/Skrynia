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
