mod group;
mod persist;
mod plan;
use super::preflight::matching;
use crate::Snapshot;
use std::{io, path::Path};
pub fn enqueue(
    data: &mut Snapshot,
    root: &Path,
    key: &str,
    kind: &str,
    max: usize,
) -> io::Result<()> {
    let ids = matching(data, key, kind);
    let updates = plan::notifications(data, ids, max)?;
    persist::apply(data, root, updates)
}
#[cfg(test)]
mod tests;

pub use group::sync_active as sync_index;
