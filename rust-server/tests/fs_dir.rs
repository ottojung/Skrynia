use skrynia_server::fs::RootKind::User as U;
use skrynia_server::{boot, fs};
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("operator")).unwrap();
    stdfs::create_dir_all(d.join("user")).unwrap();
    let code = wat::parse_str(include_str!("../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(d.join("r.wasm"), code).unwrap();
    let k = boot::Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    (d, k)
}
#[test]
fn dir_ops() {
    let (d, k) = setup("skrynia-fs-dir");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    s.mkdir(Some(U), b"x", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    for n in [b"x/a".as_slice(), b"x/bb".as_slice()] {
        let fd = s
            .open_disk(Some(U), &mut fds, n, fs::disk::DiskMode::read_write(), 0)
            .unwrap();
        s.close(&mut fds, fd).unwrap();
    }
    let fd = s.open_directory(Some(U), &mut fds, b"x", 0).unwrap();
    let mut seen = Vec::new();
    loop {
        let mut b = [0u8; 16];
        match s.next_entry(&mut fds, fd, &mut b).unwrap() {
            fs::dir::Next::Name(n) => seen.push(b[..n].to_vec()),
            fs::dir::Next::End => break,
            fs::dir::Next::Small => panic!(),
        }
    }
    seen.sort();
    assert_eq!(seen, vec![b"a".to_vec(), b"bb".to_vec()]);
    s.sync(&mut fds, fd).unwrap();
    let fd = s.open_directory(Some(U), &mut fds, b"x", 0).unwrap();
    for _ in 0..2 {
        let got = s.next_entry(&mut fds, fd, &mut []).unwrap();
        assert!(matches!(got, fs::dir::Next::Small));
    }
    let mut bytes = [0u8; 16];
    assert!(matches!(s.next_entry(&mut fds, fd, &mut bytes).unwrap(),
        fs::dir::Next::Name(n) if n > 0));
    let _ = stdfs::remove_dir_all(&d);
}
