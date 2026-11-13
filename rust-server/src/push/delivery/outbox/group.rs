use crate::{Notification, Snapshot, write_json};
use std::{collections::HashMap, io, path::Path};
pub fn path(root: &Path) -> std::path::PathBuf {
    root.join("state/push/outbox-generations.json")
}
pub fn persist(data: &Snapshot, root: &Path, updates: &[Notification]) -> io::Result<()> {
    present(root)?;
    let mut generations = data
        .push
        .outbox
        .iter()
        .map(|(id, value)| (id.clone(), value.generation))
        .collect::<HashMap<_, _>>();
    for update in updates {
        generations.insert(update.subscription_id.clone(), update.generation);
    }
    write_json(&path(root), &generations)
}

pub fn sync_active(data: &Snapshot, root: &Path) -> io::Result<()> {
    if present(root)? {
        persist(data, root, &[])?;
    }
    Ok(())
}
fn present(root: &Path) -> io::Result<bool> {
    match std::fs::symlink_metadata(path(root)) {
        Ok(info) if info.file_type().is_file() => Ok(true),
        Ok(_) => Err(io::Error::other("invalid outbox generation file")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
