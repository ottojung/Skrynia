use crate::{Result, bad};
pub fn validate(value: &serde_json::Value) -> Result<()> {
    let fields = value
        .as_object()
        .ok_or_else(|| bad("invalid_json_object"))?;
    if !fields.contains_key("endpoint") && !fields.contains_key("keys") {
        return Err(bad("nothing_to_update"));
    }
    if !fields.keys().all(|key| key == "endpoint" || key == "keys") {
        return Err(bad("invalid_subscription_patch"));
    }
    Ok(())
}
