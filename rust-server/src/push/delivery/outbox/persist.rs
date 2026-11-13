use crate::{Pending, Snapshot, persist_outbox};
use std::{io, path::Path};
pub fn apply(data: &mut Snapshot, root: &Path, updates: Vec<(String, Pending)>) -> io::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }
    let mut next = data.push.outbox.clone();
    for (id, notice) in updates {
        next.insert(id, notice);
    }
    persist_outbox(root, &next)?;
    data.push.outbox = next;
    Ok(())
}
