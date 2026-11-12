use super::disk::sync_dir;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn create(dir: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let folder = dir.join("store-tmp");
    let path = folder.join(format!("obj.{}.{}", std::process::id(), crate::random_id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o644))?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        sync_dir(&folder)?;
        Ok::<_, io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&path);
    }
    result.map(|_| path)
}
