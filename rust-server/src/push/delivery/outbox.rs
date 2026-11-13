use super::preflight::matching;
use crate::{Notification, Snapshot, sync_dir, write_json_unsynced};
use std::io;
use std::path::Path;
pub fn enqueue(
    data: &mut Snapshot,
    root: &Path,
    key: &str,
    kind: &str,
    max: usize,
) -> io::Result<()> {
    let ids = matching(data, key, kind);
    let needed = ids
        .iter()
        .filter(|id| !data.push.outbox.contains_key(*id))
        .count();
    if data.push.outbox.len().saturating_add(needed) > max {
        return Err(io::Error::other("push outbox full"));
    }
    let folder = root.join("state/push/outbox");
    let mut written = false;
    for id in ids {
        let old = data.push.outbox.get(&id);
        let generation = old.map_or(Ok(1), |pending| {
            pending
                .generation
                .checked_add(1)
                .ok_or_else(|| io::Error::other("generation overflow"))
        })?;
        let pending = Notification {
            subscription_id: id.clone(),
            generation,
            attempts: old.map_or(0, |value| value.attempts),
            next_at: old.map_or(0, |value| value.next_at),
        };
        write_json_unsynced(&folder.join(format!("{id}.json")), &pending)?;
        written = true;
        data.push.outbox.insert(id, pending);
    }
    if written {
        sync_dir(&folder)?;
    }
    Ok(())
}
