use crate::{Grant, PushConfig, PushState, endpoint_allowed, subset, valid_scope};
use std::collections::{HashMap, HashSet};
use std::io;
pub fn valid_keys(keys: &crate::PushKeys) -> bool {
    !keys.p256dh.is_empty()
        && keys.p256dh.len() <= 256
        && !keys.auth.is_empty()
        && keys.auth.len() <= 256
}
pub fn validate_push(
    state: &PushState,
    grants: &HashMap<String, Grant>,
    config: &PushConfig,
) -> io::Result<()> {
    for (id, sub) in &state.subscriptions {
        let grant = grants
            .get(&sub.grant_id)
            .ok_or_else(|| io::Error::other("orphan push subscription"))?;
        let mut seen = HashSet::new();
        if id != &sub.id
            || !valid_scope(&sub.locations)
            || !subset(&sub.locations, &grant.locations)
            || !grant.operations.iter().any(|op| op == "subscribe")
            || sub.events.is_empty()
            || sub
                .events
                .iter()
                .any(|event| crate::Kind::parse(event).is_none() || !seen.insert(event))
            || !crate::valid_sha256(&sub.cap_hash)
            || !endpoint_allowed(&sub.endpoint, config)
            || !valid_keys(&sub.keys)
        {
            return Err(io::Error::other("invalid push subscription"));
        }
    }
    for (id, pending) in &state.outbox {
        if id != &pending.subscription_id
            || pending.generation == 0
            || !state.subscriptions.contains_key(id)
        {
            return Err(io::Error::other("invalid push notification"));
        }
    }
    Ok(())
}
