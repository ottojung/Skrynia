use crate::{Fail, Result, Snapshot, matches};
use axum::http::StatusCode;
pub fn matching(data: &Snapshot, key: &str, kind: &str) -> Vec<String> {
    data.push
        .subscriptions
        .values()
        .filter(|sub| matches(&sub.locations, key) && sub.events.iter().any(|event| event == kind))
        .map(|sub| sub.id.clone())
        .collect()
}
pub fn preflight(data: &Snapshot, key: &str, kind: &str, max: usize) -> Result<()> {
    let needed = matching(data, key, kind)
        .iter()
        .filter(|id| !data.push.outbox.contains_key(*id))
        .count();
    if data.push.outbox.len().saturating_add(needed) > max {
        return Err(Fail(StatusCode::INSUFFICIENT_STORAGE, "outbox_full"));
    }
    Ok(())
}
