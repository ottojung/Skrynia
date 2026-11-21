#[cfg(unix)]
#[test]
fn startup_rejects_nonregular_root_without_blocking() {
    use skrynia_server::boot;
    use std::{ffi::CString, fs};
    let dir = std::env::temp_dir().join(format!("skrynia-root-fifo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    let fifo = dir.join("root.fifo");
    let c = CString::new(fifo.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    let result = boot::Kernel::start(&fifo, &dir.join("operator"), &dir.join("user"));
    assert!(matches!(
        result,
        Err(s) if s == "root must be a regular file"
    ));
    std::fs::remove_dir_all(dir).unwrap();
}
