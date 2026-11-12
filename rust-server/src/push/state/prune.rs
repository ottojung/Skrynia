use crate::{PushState, sync_dir};
use std::fs;
use std::io;
use std::path::Path;
pub fn orphans(root: &Path, state: &mut PushState) -> io::Result<()> {
    let dir = root.join("state/push/outbox");
    let obsolete = state
        .outbox
        .keys()
        .filter(|id| !state.subscriptions.contains_key(*id))
        .cloned()
        .collect::<Vec<_>>();
    for id in &obsolete {
        fs::remove_file(dir.join(format!("{id}.json")))?;
    }
    if !obsolete.is_empty() {
        sync_dir(&dir)?;
    }
    for id in obsolete {
        state.outbox.remove(&id);
    }
    Ok(())
}
#[cfg(test)]
mod tests;
