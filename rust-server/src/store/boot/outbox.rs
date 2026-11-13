use std::{collections::HashMap, fs, io, path::Path};
pub fn initialize(root: &Path) -> io::Result<()> {
    let file = root.join("state/push/outbox.json");
    match fs::symlink_metadata(&file) {
        Ok(meta) if meta.file_type().is_file() => Ok(()),
        Ok(_) => Err(io::Error::other("invalid outbox file")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => fresh(root, &file),
        Err(error) => Err(error),
    }
}
fn fresh(root: &Path, file: &Path) -> io::Result<()> {
    if fs::read_dir(root.join("state/grants"))?.next().is_some()
        || fs::read_dir(root.join("state/push-subs"))?.next().is_some()
    {
        return Err(io::Error::other("missing outbox file"));
    }
    crate::write_json(file, &HashMap::<String, u8>::new())
}
