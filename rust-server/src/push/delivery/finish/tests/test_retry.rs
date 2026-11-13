use super::Fixture;
use crate::{Pending, PushFailure, push_finish_batch};
#[test]
fn failed_send_retains_latest_generation_and_backoff() {
    let f = Fixture::new(2);
    let stale = Pending {
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push_finish_batch(
        &f.state,
        vec![(
            f.sub.id.clone(),
            stale,
            Some(f.sub.clone()),
            Err(PushFailure::Retry),
        )],
    )
    .unwrap();
    let pending = f.read();
    assert_eq!(pending.generation, 2);
    assert_eq!(pending.attempts, 1);
    assert!(pending.next_at >= crate::push_now());
}
