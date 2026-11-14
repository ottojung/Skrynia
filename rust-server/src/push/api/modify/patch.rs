use crate::{Fail, PushKeys, Result, Subscription, bad};
use axum::http::StatusCode;
use std::collections::HashMap;
pub fn apply(
    sub: &mut Subscription,
    value: &serde_json::Value,
    existing: &HashMap<String, Subscription>,
) -> Result<()> {
    let fields = value
        .as_object()
        .ok_or_else(|| bad("invalid_json_object"))?;
    if !fields.contains_key("endpoint") && !fields.contains_key("keys") {
        return Err(bad("nothing_to_update"));
    }
    if !fields.keys().all(|key| key == "endpoint" || key == "keys") {
        return Err(bad("invalid_subscription_patch"));
    }
    if let Some(endpoint) = fields.get("endpoint") {
        let endpoint = endpoint
            .as_str()
            .ok_or_else(|| bad("invalid_subscription"))?;
        if crate::parse_push_endpoint(endpoint).is_none() {
            return Err(bad("invalid_subscription"));
        }
        if existing.values().any(|item| {
            item.id != sub.id
                && item.endpoint == endpoint
                && item.locations == sub.locations
                && crate::same_push_events(&item.events, &sub.events)
        }) {
            return Err(Fail(StatusCode::CONFLICT, "endpoint_in_use"));
        }
        sub.endpoint = endpoint.into();
    }
    if let Some(keys) = fields.get("keys") {
        let keys: PushKeys =
            serde_json::from_value(keys.clone()).map_err(|_| bad("invalid_subscription"))?;
        if !crate::valid_push_keys(&keys) {
            return Err(bad("invalid_subscription"));
        }
        sub.keys = keys;
    }
    Ok(())
}
