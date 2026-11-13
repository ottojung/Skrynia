use crate::{Notification, PushFailure, Snapshot, State, Subscription};
pub fn retry(
    state: &State,
    data: &mut Snapshot,
    current: &Notification,
    original: &Notification,
    sub: &Subscription,
    failure: PushFailure,
) -> bool {
    let attempts = current.attempts.max(original.attempts).saturating_add(1);
    let retired = failure != PushFailure::Retry || attempts >= state.push_config.max_attempts;
    if retired
        && data
            .push
            .subscriptions
            .get(&sub.id)
            .is_some_and(|now| super::retire::unchanged(now, sub))
    {
        super::ack::remove(data, &sub.id);
        return true;
    }
    let mut next = current.clone();
    if retired {
        next.attempts = 0;
        next.next_at = 0;
    } else {
        next.attempts = attempts;
        next.next_at = crate::push_now() + crate::push_delay(attempts);
    }
    data.push.outbox.insert(sub.id.clone(), next);
    false
}
