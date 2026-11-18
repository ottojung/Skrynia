use super::FsDisk;
use std::{fs, io};
pub fn read_all(d: &FsDisk, p: &str) -> io::Result<Option<Vec<u8>>> {
    match fs::read(d.p(p)) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
pub fn write_atomic(d: &FsDisk, path: &str, bytes: &[u8]) -> io::Result<()> {
    let p = d.p(path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = p.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, p)
}
pub fn remove(d: &FsDisk, path: &str) -> io::Result<()> {
    match fs::remove_file(d.p(path)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
