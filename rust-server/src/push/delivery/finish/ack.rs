use crate::{Snapshot, sync_dir};
use std::fs;
use std::io;
use std::path::Path;
pub fn remove(root: &Path, data: &mut Snapshot, id: &str) -> io::Result<()> {
    let dir = root.join("state/push/outbox");
    let file = dir.join(format!("{id}.json"));
    if file.exists() {
        fs::remove_file(file)?;
        sync_dir(&dir)?;
    }
    data.push.outbox.remove(id);
    crate::sync_outbox_index(data, root)
}
