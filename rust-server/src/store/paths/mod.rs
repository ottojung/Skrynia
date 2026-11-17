use std::fs;
use std::path::Path;
pub fn dirs(dir: &Path) -> std::io::Result<()> {
    for (path, mode) in [
        ("store", 0o755),
        ("store-tmp", 0o700),
        ("state", 0o700),
        ("state/grants", 0o700),
        ("state/store-transactions", 0o700),
        ("state/grant-revocations", 0o700),
        ("state/push", 0o700),
        ("state/push-subs", 0o700),
    ] {
        let folder = dir.join(path);
        fs::create_dir_all(&folder)?;
        if !fs::symlink_metadata(&folder)?.file_type().is_dir() {
            return Err(std::io::Error::other(
                "storage directory must be a real directory",
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&folder, fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}
mod parents;
mod walk;
pub use parents::{ensure_parent, safe_parent};
pub use walk::{scan, visit};

#[cfg(test)]
mod tests;
