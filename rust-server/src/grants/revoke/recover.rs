use super::{disk, record::Revocation};
use std::fs;
use std::io;
use std::path::Path;

pub fn recover_pending(root: &Path) -> io::Result<()> {
    let dir = root.join("state/grant-revocations");
    fs::create_dir_all(&dir)?;
    for item in fs::read_dir(&dir)? {
        let marker = item?.path();
        let name = marker.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let record: Revocation = serde_json::from_slice(&fs::read(&marker)?)?;
        if !record.valid(name) {
            return Err(io::Error::other("invalid revocation marker"));
        }
        disk::apply(root, &marker, &record)?;
    }
    Ok(())
}
