use super::DeltaIndex;
use std::{fs, io, path::Path};
impl DeltaIndex {
    pub fn new(root: &Path) -> io::Result<Self> {
        let dir = root.join("state/scope-delta-index");
        match fs::symlink_metadata(&dir) {
            Ok(meta) if !meta.file_type().is_dir() => {
                return Err(io::Error::other("unsafe quota index directory"));
            }
            Ok(_) => fs::remove_dir_all(&dir)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        fs::create_dir(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self {
            dir,
            pending: Default::default(),
            buffer: Default::default(),
            runs: Vec::new(),
            changes: 0,
            limit: None,
        })
    }
}
