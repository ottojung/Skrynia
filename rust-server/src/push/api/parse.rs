use crate::{PushConfig, PushKeys, Result, bad};
pub struct Input {
    pub locations: String,
    pub events: Vec<String>,
    pub endpoint: String,
    pub keys: PushKeys,
}
pub fn parse(value: &serde_json::Value, config: &PushConfig) -> Result<Input> {
    let locations = value
        .get("locations")
        .and_then(|v| v.as_str())
        .ok_or_else(|| bad("invalid_subscription"))?;
    let events: Vec<String> = value.get("events").map_or(
        Ok(vec!["create".into(), "replace".into(), "delete".into()]),
        |v| serde_json::from_value(v.clone()).map_err(|_| bad("invalid_subscription")),
    )?;
    let endpoint = value
        .get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| bad("invalid_subscription"))?;
    let keys: PushKeys = serde_json::from_value(value.get("keys").cloned().unwrap_or_default())
        .map_err(|_| bad("invalid_subscription"))?;
    let mut seen = std::collections::HashSet::new();
    if !crate::valid_scope(locations)
        || !crate::endpoint_allowed(endpoint, config)
        || !crate::valid_push_keys(&keys)
        || events.is_empty()
        || events
            .iter()
            .any(|e| crate::Kind::parse(e).is_none() || !seen.insert(e))
    {
        return Err(bad("invalid_subscription"));
    }
    Ok(Input {
        locations: locations.into(),
        events,
        endpoint: endpoint.into(),
        keys,
    })
}
