use crate::{Notification, PushFailure, Snapshot, State, Subscription, write_json};
use std::io;
pub fn retry(
    state: &State,
    data: &mut Snapshot,
    current: &Notification,
    original: &Notification,
    sub: &Subscription,
    failure: PushFailure,
) -> io::Result<()> {
    let attempts = current.attempts.max(original.attempts).saturating_add(1);
    let retired = failure != PushFailure::Retry || attempts >= state.push_config.max_attempts;
    if retired
        && data
            .push
            .subscriptions
            .get(&sub.id)
            .is_some_and(|now| super::retire::unchanged(now, sub))
    {
        super::retire::remove(state, data, &sub.id)?;
        return super::ack::remove(&state.dir, data, &sub.id);
    }
    let mut next = current.clone();
    if retired {
        next.attempts = 0;
        next.next_at = 0;
    } else {
        next.attempts = attempts;
        next.next_at = crate::push_now() + crate::push_delay(attempts);
    }
    write_json(
        &state
            .dir
            .join("state/push/outbox")
            .join(format!("{}.json", sub.id)),
        &next,
    )?;
    data.push.outbox.insert(sub.id.clone(), next);
    Ok(())
}
