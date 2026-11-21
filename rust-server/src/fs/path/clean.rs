use std::path::PathBuf;

pub fn clean(bytes: &[u8]) -> Result<PathBuf, &'static str> {
    use std::os::unix::ffi::OsStrExt;
    let p = std::path::Path::new(std::ffi::OsStr::from_bytes(bytes));
    crate::safe::clean_relative(p)
}
