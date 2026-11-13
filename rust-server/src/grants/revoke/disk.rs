use super::record::Revocation;
use crate::sync_dir;
use std::{fs, io, path::Path};
mod subscriptions;
pub fn apply(root: &Path, marker: &Path, record: &Revocation) -> io::Result<()> {
    subscriptions::remove(root, record)?;
    let dir = root.join("state/grants");
    for id in record
        .ids
        .iter()
        .filter(|id| *id != &record.id)
        .chain(std::iter::once(&record.id))
    {
        let file = dir.join(format!("{id}.json"));
        if file.exists() {
            fs::remove_file(file)?;
        }
    }
    sync_dir(&dir)?;
    fs::remove_file(marker)?;
    sync_dir(marker.parent().unwrap())
}
