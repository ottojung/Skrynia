use crate::{Snapshot, State, Subscription, sync_dir};
use std::fs;
use std::io;
pub fn unchanged(a: &Subscription, b: &Subscription) -> bool {
    a.endpoint == b.endpoint
        && a.cap_hash == b.cap_hash
        && a.keys.auth == b.keys.auth
        && a.keys.p256dh == b.keys.p256dh
}
pub fn remove(state: &State, data: &mut Snapshot, id: &str) -> io::Result<()> {
    let dir = state.dir.join("state/push-subs");
    fs::remove_file(dir.join(format!("{id}.json")))?;
    sync_dir(&dir)?;
    if let Some(sub) = data.push.remove_sub(id) {
        data.change_meta(
            &sub.grant_id,
            crate::Usage {
                bytes: 4096,
                objects: 1,
            },
            false,
        );
    }
    Ok(())
}
