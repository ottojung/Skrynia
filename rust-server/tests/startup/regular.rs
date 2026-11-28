#[cfg(unix)]
#[test]
fn startup_rejects_nonregular_root_without_blocking() {
    use libskrynia::boot;
    use std::ffi::CString;
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

#[test]
fn memory64_linear_memory_works_for_buffers() {
    use libskrynia::boot::Kernel;
    let dir = std::env::temp_dir().join(format!("skrynia-m64-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    let root = dir.join("root.wasm");
    let wat = r#"(module
      (import "skrynia_v3_1" "read" (func $read (param i32 i64 i64) (result i64)))
      (import "skrynia_v3_1" "write1" (func $w (param i32 i32) (result i32)))
      (memory (export "memory") i64 1)
      (func (export "skrynia_main") (param i32 i32) (result i32)
        (drop (call $read (local.get 0) (i64.const 0) (i64.const 1)))
        (drop (call $w (local.get 1) (i32.load8_u (i64.const 0))))
        (i32.const 0)))"#;
    std::fs::write(&root, wat::parse_str(wat).unwrap()).unwrap();
    let k = Kernel::start(&root, &dir.join("operator"), &dir.join("user")).unwrap();
    assert_eq!(k.invoke(b"A").unwrap(), b"A");
    std::fs::remove_dir_all(dir).unwrap();
}
