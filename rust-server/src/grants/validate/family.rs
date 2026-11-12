use crate::{Grant, subset};
use std::collections::{HashMap, HashSet};
use std::io;
const MAX: u64 = 9007199254740991;

pub fn check(grant: &Grant, all: &HashMap<String, Grant>) -> io::Result<()> {
    if let Some(parent) = &grant.parent {
        let ancestor = all
            .get(parent)
            .ok_or_else(|| io::Error::other("orphan grant"))?;
        if !subset(&grant.locations, &ancestor.locations)
            || grant
                .operations
                .iter()
                .any(|op| !ancestor.operations.contains(op))
            || grant.max_bytes.is_none_or(|n| !(2048..=MAX).contains(&n))
            || grant.max_objects.is_none_or(|n| n == 0 || n > MAX)
            || grant.root_issuer_token.is_some()
        {
            return Err(io::Error::other("invalid grant ancestry"));
        }
    }
    let mut visited = HashSet::new();
    let mut current = Some(grant);
    while let Some(g) = current {
        if !visited.insert(&g.id) {
            return Err(io::Error::other("cyclic grant ancestry"));
        }
        current = g.parent.as_ref().and_then(|id| all.get(id));
    }
    Ok(())
}
