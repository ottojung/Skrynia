use std::{
    fs, io,
    path::{Path, PathBuf},
};
fn safe_dir(dir: &Path) -> io::Result<()> {
    let kind = fs::symlink_metadata(dir)?.file_type();
    if kind.is_symlink() {
        return Err(io::Error::other("unsafe store symlink"));
    }
    if !kind.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "object is a parent directory",
        ));
    }
    Ok(())
}
pub fn safe_parent(root: &Path, key: &str) -> io::Result<PathBuf> {
    let store = root.join("store");
    safe_dir(&store)?;
    let mut parent = store;
    let components: Vec<_> = key.split('/').collect();
    for part in &components[..components.len() - 1] {
        parent.push(part);
        safe_dir(&parent)?;
    }
    Ok(parent)
}
pub fn ensure_parent(root: &Path, key: &str) -> io::Result<PathBuf> {
    let store = root.join("store");
    safe_dir(&store)?;
    let mut parent = store;
    let components: Vec<_> = key.split('/').collect();
    for part in &components[..components.len() - 1] {
        let next = parent.join(part);
        match fs::create_dir(&next) {
            Ok(()) => {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&next, fs::Permissions::from_mode(0o755))?;
                crate::sync_dir(&parent)?;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        safe_dir(&next)?;
        parent = next;
    }
    Ok(parent)
}
