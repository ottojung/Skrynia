use crate::{Result, State, missing, sync_dir};
use std::fs;
pub fn remove(state: &State, id: &str, cap: &str) -> Result<()> {
    let mut data = state.data.lock().map_err(|_| crate::internal())?;
    let sub = data
        .push
        .subscriptions
        .get(id)
        .ok_or_else(|| missing("not_found"))?;
    super::require_capability(&sub.cap_hash, cap)?;
    fs::remove_file(state.dir.join("state/push-subs").join(format!("{id}.json")))
        .map_err(|_| crate::internal())?;
    sync_dir(&state.dir.join("state/push-subs")).map_err(|_| crate::internal())?;
    data.push.subscriptions.remove(id);
    Ok(())
}
