use super::Revocation;
use crate::sync_dir;
use std::{fs, io, path::Path};
pub fn remove(root: &Path, record: &Revocation) -> io::Result<()> {
    let directory = root.join("state/push-subs");
    let mut affected = Vec::new();
    for item in fs::read_dir(&directory)? {
        let file = item?.path();
        let sub: crate::Subscription = serde_json::from_slice(&fs::read(&file)?)?;
        if record.ids.contains(&sub.grant_id) {
            affected.push((sub.id, file));
        }
    }
    if !affected.is_empty() {
        let mut outbox = crate::read_outbox(root)?;
        for (id, _) in &affected {
            outbox.remove(id);
        }
        crate::persist_outbox(root, &outbox)?;
        for (_, file) in affected {
            fs::remove_file(file)?;
        }
        sync_dir(&directory)?;
    }
    Ok(())
}
