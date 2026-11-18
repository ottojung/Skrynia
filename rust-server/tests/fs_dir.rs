use skrynia_server::{boot, fs};
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("data")).unwrap();
    stdfs::write(d.join("r.wasm"), b"\0asm\x01\0\0\0").unwrap();
    let k = boot::Kernel::start(&d.join("r.wasm"), &d.join("data")).unwrap();
    (d, k)
}
#[test]
fn dir_ops() {
    let (d, k) = setup("skrynia-fs-dir");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    s.mkdir(b"x", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    for n in [b"x/a".as_slice(), b"x/bb".as_slice()] {
        let fd = s
            .open_disk(&mut fds, n, fs::disk::DiskMode::read_write())
            .unwrap();
        s.close(&mut fds, fd).unwrap();
    }
    let fd = s.open_directory(&mut fds, b"x").unwrap();
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
    s.close(&mut fds, fd).unwrap();
    let fd = s.open_directory(&mut fds, b"x").unwrap();
    assert!(matches!(
        s.next_entry(&mut fds, fd, &mut [0u8; 1]).unwrap(),
        fs::dir::Next::Small
    ));
    let _ = stdfs::remove_dir_all(&d);
}
