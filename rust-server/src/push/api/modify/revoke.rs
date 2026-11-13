use crate::{Result, State, missing};
pub fn remove(state: &State, id: &str, cap: &str) -> Result<()> {
    let mut data = state.data.lock().map_err(|_| crate::internal())?;
    state.require_healthy()?;
    let sub = data
        .push
        .subscriptions
        .get(id)
        .ok_or_else(|| missing("not_found"))?;
    super::require_capability(&sub.cap_hash, cap)?;
    state.persist_remove(&state.dir.join("state/push-subs").join(format!("{id}.json")))?;
    data.push.subscriptions.remove(id);
    let outbox = state.dir.join("state/push/outbox");
    let notice = outbox.join(format!("{id}.json"));
    if notice.exists() {
        state.persist_remove(&notice)?;
    }
    data.push.outbox.remove(id);
    Ok(())
}
