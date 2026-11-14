use crate::{Pending, PushState};
#[test]
fn clean_ack_backoff_is_volatile_and_exponential() {
    let mut push = PushState::default();
    let id = "subscriber";
    let pending = Pending {
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push.outbox.insert(id.into(), pending.clone());
    let now = crate::push_now();
    push.delay_failed_ack(id, false);
    assert_eq!(push.retry_attempts(id, &pending), 0);
    assert!(!push.ready(id, &pending, now + 900));
    assert!(push.ready(id, &pending, now + 1500));
    push.delay_failed_ack(id, false);
    assert!(!push.ready(id, &pending, now + 1500));
    assert!(push.ready(id, &pending, now + 2500));
    push.delay_failed_ack(id, false);
    assert!(!push.ready(id, &pending, now + 3000));
    assert_eq!(push.effective_retry(id, &pending), (0, 0));
}
#[test]
fn failed_deliveries_accumulate_and_survive_new_generations() {
    let mut push = PushState::default();
    let id = "subscriber";
    let pending = Pending {
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push.outbox.insert(id.into(), pending.clone());
    for expected in 1..=3 {
        push.delay_failed_ack(id, true);
        assert_eq!(push.retry_attempts(id, &pending), expected);
    }
    let (attempts, deadline) = push.effective_retry(id, &pending);
    assert_eq!(attempts, 3);
    assert!(deadline > crate::push_now());
    push.outbox.get_mut(id).unwrap().generation = 2;
    push.prune_cooldowns();
    assert!(push.ready(id, push.outbox.get(id).unwrap(), crate::push_now()));
}
