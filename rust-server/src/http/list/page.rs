use super::{cursor, params::Params};
use crate::{Result, Snapshot, forbidden, subset};

pub fn list(state: &Snapshot, token: &str, request: Params) -> Result<serde_json::Value> {
    let grant = state.user(token)?;
    if !grant.operations.iter().any(|x| x == "list") {
        return Err(forbidden("grant_operation_forbidden"));
    }
    if !subset(&request.locations, &grant.locations) {
        return Err(forbidden("locations_outside_grant"));
    }
    let keys = cursor::entries(state, &request);
    if keys.len() > request.limit {
        let entries = keys[..request.limit].to_vec();
        Ok(serde_json::json!({"keys":entries,"next":entries.last()}))
    } else {
        Ok(serde_json::json!({"keys":keys}))
    }
}
