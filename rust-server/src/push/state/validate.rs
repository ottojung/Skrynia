pub use super::keys::valid as valid_keys;
use crate::{Grant, PushState, subset, valid_scope};
use std::collections::{HashMap, HashSet};
use std::io;
pub fn validate_push(state: &PushState, grants: &HashMap<String, Grant>) -> io::Result<()> {
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
            || crate::parse_push_endpoint(&sub.endpoint).is_none()
            || !super::keys::legacy(&sub.keys)
        {
            return Err(io::Error::other("invalid push subscription"));
        }
        if !valid_keys(&sub.keys) {
            eprintln!(
                "Skrynia subscription {id} has legacy invalid push keys; re-register to restore delivery"
            );
        }
    }
    for pending in state.outbox.values() {
        if pending.generation == 0 {
            return Err(io::Error::other("invalid push notification"));
        }
    }
    Ok(())
}
