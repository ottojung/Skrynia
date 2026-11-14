mod attach;
use super::spec::{self, Spec};
use crate::{Result, Snapshot, State, Usage};
pub use attach::attach;
use std::sync::MutexGuard;
pub fn checked<'a>(
    state: &'a State,
    token: &str,
    value: &serde_json::Value,
) -> Result<(MutexGuard<'a, Snapshot>, Spec, Option<Usage>)> {
    for _ in 0..2 {
        let data = state.data.lock().map_err(|_| crate::internal())?;
        state.require_healthy()?;
        let parent = data.user(token)?;
        let spec = spec::parse(parent, value)?;
        data.check_metadata(&parent.id, 2048)?;
        if data.quota.scopes.has(&spec.locations) {
            return Ok((data, spec, None));
        }
        let revision = data.object_version;
        drop(data);
        let total = crate::QuotaIndex::scope_usage(&state.dir, &spec.locations)
            .map_err(|_| crate::internal())?;
        let data = state.data.lock().map_err(|_| crate::internal())?;
        state.require_healthy()?;
        if data.object_version != revision {
            continue;
        }
        let parent = data.user(token)?;
        let spec = spec::parse(parent, value)?;
        data.check_metadata(&parent.id, 2048)?;
        let total = (!data.quota.scopes.has(&spec.locations)).then_some(total);
        return Ok((data, spec, total));
    }
    // Make progress under sustained writes; the scan must see one snapshot.
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let parent = data.user(token)?;
    let spec = spec::parse(parent, value)?;
    data.check_metadata(&parent.id, 2048)?;
    if data.quota.scopes.has(&spec.locations) {
        return Ok((data, spec, None));
    }
    let total = crate::QuotaIndex::scope_usage(&state.dir, &spec.locations)
        .map_err(|_| crate::internal())?;
    Ok((data, spec, Some(total)))
}
