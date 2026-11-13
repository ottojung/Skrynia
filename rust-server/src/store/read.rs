use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
// Do not follow a link introduced after startup; do not block on named pipes.
pub fn open_regular(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("stored object is not a regular file"));
    }
    Ok(file)
}
#[cfg(test)]
mod tests {
    use super::open_regular;
    use std::fs;
    #[test]
    fn regular_file_opens_but_symlink_to_private_data_never_does() {
        let root = std::env::temp_dir().join(format!("skrynia-safe-read-{}", crate::random_id()));
        fs::create_dir(&root).unwrap();
        let object = root.join("object");
        let linked = root.join("linked");
        fs::write(&object, b"public bytes").unwrap();
        std::os::unix::fs::symlink(&object, &linked).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut open_regular(&object).unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, b"public bytes");
        assert!(open_regular(&linked).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn named_pipe_is_rejected_without_blocking_for_a_writer() {
        use std::os::unix::ffi::OsStrExt;
        let path = std::env::temp_dir().join(format!("skrynia-fifo-{}", crate::random_id()));
        let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        assert!(open_regular(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
