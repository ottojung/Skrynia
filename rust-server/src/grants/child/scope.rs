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
    let data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let parent = data.user(token)?;
    let spec = spec::parse(parent, value)?;
    data.check_metadata(&parent.id, 2048)?;
    let total = if data.quota.scopes.has(&spec.locations) {
        None
    } else if let Some(index) = &data.quota.delta {
        let result = index
            .lock()
            .map_err(|_| crate::internal())?
            .scope(&state.dir, &spec.locations);
        Some(result.map_err(|error| {
            state
                .failed
                .store(true, std::sync::atomic::Ordering::Release);
            eprintln!("Skrynia quota index query failed: {error}");
            crate::internal()
        })?)
    } else {
        Some(
            crate::QuotaIndex::scope_usage(&state.dir, &spec.locations)
                .map_err(|_| crate::internal())?,
        )
    };
    Ok((data, spec, total))
}
