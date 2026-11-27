use libskrynia::safe;
use std::path::Path;

#[test]
fn rejects_bad_paths() {
    for p in ["/abs", "../x", "a/../../b", "a\0b", "", "a//b", "a/"] {
        assert!(safe::clean_relative(Path::new(p)).is_err(), "{p}");
    }
    use std::os::unix::ffi::OsStrExt;
    let bad = std::ffi::OsStr::from_bytes(b"a/\xff");
    assert!(safe::clean_relative(Path::new(bad)).is_err());
}

#[test]
fn accepts_normal_paths() {
    let p = safe::clean_relative(Path::new("a/b/c.txt")).unwrap();
    assert_eq!(p, Path::new("a/b/c.txt"));
    let j = safe::join(Path::new("/data"), Path::new("x/y")).unwrap();
    assert_eq!(j, Path::new("/data/x/y"));
}

#[test]
fn time_and_random_look_valid() {
    assert!(libskrynia::sys::clock::now_millis() > 1_700_000_000_000);
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    libskrynia::sys::rand::fill_secure(&mut a).unwrap();
    libskrynia::sys::rand::fill_secure(&mut b).unwrap();
    assert_ne!(a, [0u8; 32]);
    assert_ne!(a, b);
}
