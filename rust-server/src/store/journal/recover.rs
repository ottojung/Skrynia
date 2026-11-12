use super::disk::sync_dir;
use super::intent::Intent;
use std::fs;
use std::io;
use std::path::Path;

pub fn recover(root: &Path) -> io::Result<()> {
    for directory in [
        "state/push-subs",
        "state/push/outbox",
        "state/grant-revocations",
    ] {
        let folder = root.join(directory);
        if folder.exists() && fs::read_dir(folder)?.next().transpose()?.is_some() {
            return Err(io::Error::other(format!(
                "unsupported pending state: {directory}"
            )));
        }
    }
    super::cleanup::cleanup_temp(root)?;
    let dir = root.join("state/store-transactions");
    for item in fs::read_dir(&dir)? {
        let file = item?.path();
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = name.strip_suffix(".json").unwrap_or("");
        if stem.len() != 32
            || !stem
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(io::Error::other("invalid transaction filename"));
        }
        let intent: Intent = serde_json::from_slice(&fs::read(&file)?)?;
        if !intent.valid() {
            return Err(io::Error::other("invalid transaction record"));
        }
        // Disk state is authoritative; no notification subscribers are permitted yet.
        let _ = intent.applied(root)?;
        fs::remove_file(file)?;
    }
    sync_dir(&dir)
}
