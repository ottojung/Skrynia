use super::{cursor, params::Params};
use crate::Result;
use std::path::Path;
pub fn list(request: Params, dir: &Path) -> Result<serde_json::Value> {
    let keys = cursor::entries(dir, &request).map_err(|_| crate::internal())?;
    if keys.len() > request.limit {
        let entries = keys[..request.limit].to_vec();
        Ok(serde_json::json!({"keys":entries,"next":entries.last()}))
    } else {
        Ok(serde_json::json!({"keys":keys}))
    }
}
