use libskrynia::fs::RootKind::User as U;
use libskrynia::{boot, fs};
use std::fs as stdfs;
fn setup(name: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(name);
    let _ = stdfs::remove_dir_all(&dir);
    stdfs::create_dir_all(dir.join("operator")).unwrap();
    stdfs::create_dir_all(dir.join("user")).unwrap();
    let wasm = dir.join("r.wasm");
    let code = wat::parse_str(include_str!("../../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(&wasm, code).unwrap();
    let k = boot::Kernel::start(&wasm, &dir.join("operator"), &dir.join("user")).unwrap();
    (dir, k)
}
#[test]
fn rejects_bad_and_escaping_paths() {
    let (dir, k) = setup("skrynia-fs-node2");
    let s = k.fs();
    for p in [
        b"/abs".as_slice(),
        b"user/../x",
        b"user/a/../../b",
        b"a\0b",
        b"",
    ] {
        assert!(s.exists(Some(U), p).is_err());
    }
    let mut fds = fs::table::Table::new();
    assert!(
        s.open_disk(
            Some(U),
            &mut fds,
            b"user/../out",
            fs::disk::DiskMode::read_write(),
            0,
        )
        .is_err()
    );
    let _ = stdfs::remove_dir_all(&dir);
}
