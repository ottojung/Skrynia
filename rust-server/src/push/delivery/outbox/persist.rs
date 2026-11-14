use crate::{Pending, Snapshot, persist_outbox};
use std::{io, path::Path};
pub fn apply(data: &mut Snapshot, root: &Path, updates: Vec<(String, Pending)>) -> io::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }
    let previous = updates
        .iter()
        .map(|(id, _)| (id.clone(), data.push.outbox.get(id).cloned()))
        .collect::<Vec<_>>();
    for (id, notice) in updates {
        data.push.outbox.insert(id, notice);
    }
    if let Err(error) = persist_outbox(root, &data.push.outbox) {
        for (id, old) in previous {
            if let Some(notice) = old {
                data.push.outbox.insert(id, notice);
            } else {
                data.push.outbox.remove(&id);
            }
        }
        return Err(error);
    }
    Ok(())
}
