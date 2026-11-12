use crate::{Notification, State, Subscription};
pub type Entry = (Notification, Option<Subscription>);
pub fn due(state: &State) -> Vec<Entry> {
    let Ok(data) = state.data.lock() else {
        return Vec::new();
    };
    let now = crate::push_now();
    data.push
        .outbox
        .values()
        .filter(|entry| entry.next_at <= now)
        .map(|entry| {
            (
                entry.clone(),
                data.push.subscriptions.get(&entry.subscription_id).cloned(),
            )
        })
        .collect()
}
