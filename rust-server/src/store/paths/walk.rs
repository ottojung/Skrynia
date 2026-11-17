use std::{fs, io, path::Path};
pub fn visit(root: &Path, mut f: impl FnMut(&str, u64) -> io::Result<()>) -> io::Result<()> {
    scan_dir(&root.join("store"), "", &mut f)
}
fn scan_dir(
    dir: &Path,
    prefix: &str,
    f: &mut impl FnMut(&str, u64) -> io::Result<()>,
) -> io::Result<()> {
    if !fs::symlink_metadata(dir)?.file_type().is_dir() {
        return Err(io::Error::other("unsafe store directory"));
    }
    for item in fs::read_dir(dir)? {
        let item = item?;
        let part = item
            .file_name()
            .into_string()
            .map_err(|_| io::Error::other("bad key"))?;
        let key = format!("{prefix}{part}");
        if !crate::valid_key(&key) {
            return Err(io::Error::other("invalid stored key"));
        }
        let kind = item.file_type()?;
        if kind.is_dir() {
            scan_dir(&item.path(), &format!("{key}/"), f)?;
        } else if kind.is_file() {
            f(&key, item.metadata()?.len())?;
        } else {
            return Err(io::Error::other("nonregular stored object"));
        }
    }
    Ok(())
}
pub fn scan(root: &Path) -> io::Result<()> {
    visit(root, |_, _| Ok(()))
}
