mod patch;
mod revoke;
use crate::{Result, State, missing};
impl State {
    pub fn update_subscription(
        &self,
        id: &str,
        cap: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        let mut sub = data
            .push
            .subscriptions
            .get(id)
            .cloned()
            .ok_or_else(|| missing("not_found"))?;
        require_capability(&sub.cap_hash, cap)?;
        patch::apply(&mut sub, value, &data.push.subscriptions, &self.push_config)?;
        crate::persist_push_subscription(self, &sub)?;
        data.push.subscriptions.insert(id.into(), sub);
        Ok(())
    }
    pub fn remove_subscription(&self, id: &str, cap: &str) -> Result<()> {
        revoke::remove(self, id, cap)
    }
}
pub fn require_capability(expected: &str, provided: &str) -> Result<()> {
    if !crate::secure_match(expected, &crate::sha(provided.as_bytes())) {
        return Err(crate::forbidden("invalid_capability"));
    }
    Ok(())
}
