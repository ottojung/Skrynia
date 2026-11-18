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
