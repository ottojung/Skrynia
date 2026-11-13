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
    if data.push.outbox.contains_key(id) {
        let mut next = data.push.outbox.clone();
        next.remove(id);
        state.fail_on_io(crate::persist_outbox(&state.dir, &next))?;
        data.push.outbox = next;
    }
    state.persist_remove(&state.dir.join("state/push-subs").join(format!("{id}.json")))?;
    if let Some(sub) = data.push.remove_sub(id) {
        data.change_meta(
            &sub.grant_id,
            crate::Usage {
                bytes: 4096,
                objects: 1,
            },
            false,
        );
    }
    Ok(())
}
