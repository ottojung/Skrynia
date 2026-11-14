mod fields;
mod limits;
use crate::{Grant, Result, bad, forbidden, subset, valid_scope};

pub struct Spec {
    pub locations: String,
    pub operations: Vec<String>,
    pub max_bytes: u64,
    pub max_objects: u64,
}
pub fn parse(parent: &Grant, v: &serde_json::Value) -> Result<Spec> {
    if !parent.operations.iter().any(|op| op == "delegate") {
        return Err(forbidden("grant_operation_forbidden"));
    }
    let locations = fields::locations(v)?;
    if !valid_scope(locations) {
        return Err(bad("invalid_locations"));
    }
    if !subset(locations, &parent.locations) {
        return Err(forbidden("locations_outside_parent"));
    }
    let operations = fields::operations(v)?;
    let mut seen = std::collections::HashSet::new();
    if operations
        .iter()
        .any(|op| !crate::OPERATIONS.contains(&op.as_str()) || !seen.insert(op))
    {
        return Err(bad("invalid_operations"));
    }
    if operations.iter().any(|op| !parent.operations.contains(op)) {
        return Err(forbidden("operations_outside_parent"));
    }
    let (max_bytes, max_objects) = limits::parse(v)?;
    Ok(Spec {
        locations: locations.into(),
        operations,
        max_bytes,
        max_objects,
    })
}
