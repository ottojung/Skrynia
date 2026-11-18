use super::stream::Stream;
use std::{
    fs,
    os::{fd::OwnedFd, unix::ffi::OsStrExt},
};
#[test]
fn malformed_names_fail_without_lossy_conversion() {
    let path = std::env::temp_dir().join(format!("skrynia-v3-dir-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let invalid = std::ffi::OsStr::from_bytes(b"\xff");
    fs::write(path.join(invalid), b"").unwrap();
    let fd = OwnedFd::from(fs::File::open(&path).unwrap());
    let mut it = Stream::open(&fd).unwrap();
    assert_eq!(it.next().err(), Some("invalid"));
    drop(it);
    fs::remove_dir_all(path).unwrap();
}
