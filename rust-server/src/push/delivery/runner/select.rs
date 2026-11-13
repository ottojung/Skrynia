use crate::{Pending, State, Subscription};
pub type Entry = (String, Pending, Option<Subscription>);
pub fn due(state: &State) -> Vec<Entry> {
    let Ok(data) = state.data.lock() else {
        return Vec::new();
    };
    let now = crate::push_now();
    data.push
        .outbox
        .iter()
        .filter(|(_, entry)| entry.next_at <= now)
        .map(|(id, entry)| {
            (
                id.clone(),
                entry.clone(),
                data.push.subscriptions.get(id).cloned(),
            )
        })
        .collect()
}
