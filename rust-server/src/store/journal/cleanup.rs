use super::disk::sync_dir;
use std::fs;
use std::io;
use std::path::Path;
pub fn cleanup_temp(root: &Path) -> io::Result<()> {
    let dir = root.join("store-tmp");
    for item in fs::read_dir(&dir)? {
        let file = item?.path();
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let mut parts = name.split('.');
        if parts.next() == Some("obj")
            && parts.next().is_some_and(|v| v.parse::<u32>().is_ok())
            && parts.next().is_some_and(|v| {
                (v.len() == 24 || v.len() == 32) && v.bytes().all(|c| c.is_ascii_hexdigit())
            })
            && parts.next().is_none()
        {
            fs::remove_file(file)?;
        }
    }
    sync_dir(&dir)
}
