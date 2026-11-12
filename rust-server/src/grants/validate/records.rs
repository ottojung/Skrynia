use super::identifiers::{valid_hash, valid_id};
use crate::{Grant, OPERATIONS, valid_scope};
use std::collections::HashSet;
use std::io;

pub fn check(grant: &Grant, seen: &mut HashSet<String>) -> io::Result<()> {
    let mut ops = HashSet::new();
    if !valid_id(&grant.id)
        || !valid_hash(&grant.issuer_hash)
        || !valid_scope(&grant.locations)
        || grant
            .operations
            .iter()
            .any(|op| !OPERATIONS.contains(&op.as_str()) || !ops.insert(op))
    {
        return Err(io::Error::other("invalid grant record"));
    }
    for user in &grant.users {
        if !valid_id(&user.id) || !valid_hash(&user.hash) || !seen.insert(user.hash.clone()) {
            return Err(io::Error::other("invalid grant user"));
        }
    }
    if grant.parent.is_none()
        && (grant.locations != "*"
            || grant.max_bytes.is_some()
            || grant.max_objects.is_some()
            || grant.operations.len() != OPERATIONS.len()
            || grant
                .root_issuer_token
                .as_deref()
                .is_none_or(|t| !valid_hash(t) || crate::sha(t.as_bytes()) != grant.issuer_hash))
    {
        return Err(io::Error::other("invalid root grant"));
    }
    Ok(())
}
