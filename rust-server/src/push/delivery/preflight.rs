use crate::{Fail, Result, Snapshot};
use axum::http::StatusCode;
pub fn matching(data: &Snapshot, key: &str, kind: &str) -> Vec<String> {
    data.push.index.matching(key, kind)
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
