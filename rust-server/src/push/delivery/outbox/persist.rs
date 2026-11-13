use super::group;
use crate::{Notification, Snapshot, sync_dir, write_json_unsynced};
use std::{io, path::Path};
pub fn apply(data: &mut Snapshot, root: &Path, updates: Vec<Notification>) -> io::Result<()> {
    let grouped = updates.len() > 1;
    if grouped {
        group::persist(data, root, &updates)?;
    }
    let folder = root.join("state/push/outbox");
    for notice in updates {
        let id = notice.subscription_id.clone();
        if !grouped {
            write_json_unsynced(&folder.join(format!("{id}.json")), &notice)?;
            sync_dir(&folder)?;
        }
        data.push.outbox.insert(id, notice);
    }
    Ok(())
}
