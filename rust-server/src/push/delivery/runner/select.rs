use crate::{Pending, State, Subscription};
pub type Entry = (String, Pending, Option<Subscription>);
pub fn due(state: &State) -> Vec<Entry> {
    let Ok(mut data) = state.data.lock() else {
        return Vec::new();
    };
    data.push.prune_cooldowns();
    let now = crate::push_now();
    data.push
        .outbox
        .iter()
        .filter(|(id, entry)| data.push.ready(id, entry, now))
        .map(|(id, entry)| {
            (
                id.clone(),
                entry.clone(),
                data.push.subscriptions.get(id).cloned(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::due;
    use crate::Fixture;
    #[test]
    fn volatile_retry_deadline_suppresses_queued_delivery() {
        let fixture = Fixture::new(1);
        assert_eq!(due(&fixture.state).len(), 1);
        fixture
            .state
            .data
            .lock()
            .unwrap()
            .push
            .delay_failed_ack(&fixture.sub.id, false);
        assert!(due(&fixture.state).is_empty());
    }
}
