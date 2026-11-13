use crate::{Notification, Snapshot, persist_outbox};
use std::{io, path::Path};
pub fn apply(data: &mut Snapshot, root: &Path, updates: Vec<Notification>) -> io::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }
    let mut next = data.push.outbox.clone();
    for notice in updates {
        next.insert(notice.subscription_id.clone(), notice);
    }
    persist_outbox(root, &next)?;
    data.push.outbox = next;
    Ok(())
}
