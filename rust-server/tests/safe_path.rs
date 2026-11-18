use skrynia_server::safe;
use std::path::Path;

#[test]
fn rejects_bad_paths() {
    assert!(safe::clean_relative(Path::new("/abs")).is_err());
    assert!(safe::clean_relative(Path::new("../x")).is_err());
    assert!(safe::clean_relative(Path::new("a/../../b")).is_err());
    assert!(safe::clean_relative(Path::new("a\0b")).is_err());
    assert!(safe::clean_relative(Path::new("")).is_err());
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
    assert!(skrynia_server::sys::clock::now_millis() > 1_700_000_000_000);
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    skrynia_server::sys::rand::fill_secure(&mut a).unwrap();
    skrynia_server::sys::rand::fill_secure(&mut b).unwrap();
    assert_ne!(a, [0u8; 32]);
    assert_ne!(a, b);
}
