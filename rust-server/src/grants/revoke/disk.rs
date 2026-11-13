use super::record::Revocation;
use crate::sync_dir;
use std::fs;
use std::io;
use std::path::Path;

pub fn apply(root: &Path, marker: &Path, record: &Revocation) -> io::Result<()> {
    let subdir = root.join("state/push-subs");
    let mut sub_ids = Vec::new();
    if subdir.exists() {
        for item in fs::read_dir(&subdir)? {
            let file = item?.path();
            let value: serde_json::Value = serde_json::from_slice(&fs::read(&file)?)?;
            let grant = value.get("grantId").and_then(|v| v.as_str()).unwrap_or("");
            if record.ids.iter().any(|id| id == grant) {
                fs::remove_file(&file)?;
                if let Some(id) = value.get("id").and_then(|v| v.as_str())
                    && crate::valid_identifier(id)
                {
                    sub_ids.push(id.to_string());
                }
            }
        }
        sync_dir(&subdir)?;
    }
    let outbox = root.join("state/push/outbox");
    for id in sub_ids {
        let file = outbox.join(format!("{id}.json"));
        if file.exists() {
            fs::remove_file(file)?;
        }
    }
    sync_dir(&outbox)?;
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
