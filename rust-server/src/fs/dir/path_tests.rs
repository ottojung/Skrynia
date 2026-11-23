use std::ffi::CString;
use std::os::fd::AsRawFd;

#[test]
fn missing_final_or_intermediate_directory_is_not_an_io_failure() {
    let d = std::env::temp_dir().join(format!("skrynia-fs-enoent-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("present")).unwrap();
    let root = std::fs::File::open(&d).unwrap();
    for name in ["absent", "absent/child", "present/absent"] {
        let parts: Vec<CString> = name.split('/').map(|s| CString::new(s).unwrap()).collect();
        assert_eq!(
            super::path::open_path(root.as_raw_fd(), &parts).unwrap_err(),
            "missing"
        );
    }
    std::fs::remove_dir_all(&d).unwrap();
}

#[test]
fn nondirectory_path_remains_a_real_failure_not_missing() {
    let d = std::env::temp_dir().join(format!("skrynia-fs-enotdir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("file"), b"x").unwrap();
    let root = std::fs::File::open(&d).unwrap();
    let parts = [
        CString::new("file").unwrap(),
        CString::new("child").unwrap(),
    ];
    assert_eq!(
        super::path::open_path(root.as_raw_fd(), &parts).unwrap_err(),
        "open"
    );
    std::fs::remove_dir_all(&d).unwrap();
}
