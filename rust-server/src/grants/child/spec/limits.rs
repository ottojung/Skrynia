use crate::{Result, bad};
pub fn parse(value: &serde_json::Value) -> Result<(u64, u64)> {
    let max_bytes = value
        .get("maxBytes")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| bad("invalid_grant_limits"))?;
    let max_objects = value
        .get("maxObjects")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| bad("invalid_grant_limits"))?;
    let safe_max = 9007199254740991u64;
    if max_bytes < 2048 || max_objects == 0 || max_bytes > safe_max || max_objects > safe_max {
        return Err(bad("invalid_grant_limits"));
    }
    Ok((max_bytes, max_objects))
}
