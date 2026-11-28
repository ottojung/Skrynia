use std::path::{Component, Path, PathBuf};

pub(crate) const MAX_RELATIVE_PATH_BYTES: usize = 4096;

pub fn clean_relative(path: &Path) -> Result<PathBuf, &'static str> {
    if path.as_os_str().is_empty() {
        return Err("empty");
    }
    let s = path.as_os_str().as_encoded_bytes();
    if s.contains(&0) {
        return Err("null");
    }
    if std::str::from_utf8(s).is_err() || s.windows(2).any(|w| w == b"//") || s.ends_with(b"/") {
        return Err("invalid");
    }
    if s.len() > MAX_RELATIVE_PATH_BYTES {
        return Err("invalid");
    }
    if path.is_absolute() {
        return Err("absolute");
    }
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("escape");
            }
        }
    }
    if out.as_os_str().is_empty() {
        return Err("empty");
    }
    let out_len = out.as_os_str().as_encoded_bytes().len();
    if out_len > MAX_RELATIVE_PATH_BYTES {
        return Err("invalid");
    }
    Ok(out)
}

pub fn join(base: &Path, rel: &Path) -> Result<PathBuf, &'static str> {
    Ok(base.join(clean_relative(rel)?))
}
