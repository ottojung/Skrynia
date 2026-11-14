use crate::{Result, bad};
pub fn locations(v: &serde_json::Value) -> Result<&str> {
    let object = v.as_object().ok_or_else(|| bad("invalid_grant_spec"))?;
    if object
        .keys()
        .any(|k| !["locations", "operations", "maxBytes", "maxObjects"].contains(&k.as_str()))
    {
        return Err(bad("invalid_grant_spec"));
    }
    object
        .get("locations")
        .and_then(|v| v.as_str())
        .ok_or_else(|| bad("invalid_locations"))
}
pub fn operations(v: &serde_json::Value) -> Result<Vec<String>> {
    serde_json::from_value(
        v.get("operations")
            .cloned()
            .ok_or_else(|| bad("invalid_operations"))?,
    )
    .map_err(|_| bad("invalid_operations"))
}
