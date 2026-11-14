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
pub fn scan(dir: &Path) -> std::io::Result<()> {
    for item in fs::read_dir(dir.join("store"))? {
        let item = item?;
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| std::io::Error::other("bad key"))?;
        if !crate::valid_key(&name) || !item.file_type()?.is_file() {
            return Err(std::io::Error::other("invalid stored key"));
        }
    }
    Ok(())
}
