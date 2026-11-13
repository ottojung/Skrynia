use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    File::open(dir)?.sync_all()
}
pub fn write_atomic(path: &Path, bytes: &[u8], private: bool) -> std::io::Result<()> {
    write_atomic_inner(path, bytes, private, true)
}
fn write_atomic_inner(
    path: &Path,
    bytes: &[u8],
    private: bool,
    sync_parent: bool,
) -> std::io::Result<()> {
    let parent = path.parent().unwrap();
    let tmp = parent.join(format!(".skrynia-{}", crate::random_id()));
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(if private {
                0o600
            } else {
                0o644
            }))?;
        }
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp, path)?;
        if sync_parent {
            sync_dir(parent)
        } else {
            Ok(())
        }
    })();
    let _ = fs::remove_file(&tmp);
    result
}
pub fn write_json(path: &Path, value: &impl serde::Serialize) -> std::io::Result<()> {
    write_atomic(path, &serde_json::to_vec(value)?, true)
}

pub fn write_json_unsynced(path: &Path, value: &impl serde::Serialize) -> std::io::Result<()> {
    write_atomic_inner(path, &serde_json::to_vec(value)?, true, false)
}
