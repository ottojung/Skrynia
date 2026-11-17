use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
pub struct Staged(PathBuf);
impl Staged {
    pub fn start(dir: &Path) -> io::Result<(Self, File)> {
        let folder = dir.join("store-tmp");
        let path = folder.join(format!("obj.{}.{}", std::process::id(), crate::random_id()));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(error) = file.set_permissions(fs::Permissions::from_mode(0o644)) {
                let _ = fs::remove_file(&path);
                return Err(error);
            }
        }
        Ok((Self(path), file))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Staged {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
