#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{io::Read, path::Path};
pub fn bytes(path: &Path) -> Result<Vec<u8>, String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NONBLOCK);
    let mut file = options.open(path).map_err(|e| format!("open root: {e}"))?;
    let meta = file.metadata().map_err(|e| format!("stat root: {e}"))?;
    if !meta.is_file() {
        return Err("root must be a regular file".into());
    }
    let limit = super::vm::state::MAX_MODULE_BYTES;
    if meta.len() > limit as u64 {
        return Err("root module too large".into());
    }
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read root: {e}"))?;
    if bytes.len() > limit {
        return Err("root module too large".into());
    }
    Ok(bytes)
}
