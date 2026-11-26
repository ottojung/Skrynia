use super::support::setup;
use libskrynia::fs::{self, RootKind};

#[test]
fn namespace_escape_and_malformed_tokens_are_rejected() {
    let (_dir, k) = setup("k10-escape");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    for p in [
        b"user/../operator/k".as_slice(),
        b"operator/../user/k",
        b"user//k",
        b"user/",
        b"user",
        b"operator",
        b"user/k\0.bin",
        b"/user/k",
        b"user/./../operator/k",
    ] {
        assert!(s.exists(None, p).is_err(), "{p:?}");
        assert!(s.open_directory(None, &mut fds, p).is_err(), "{p:?}");
    }
}

#[test]
fn root_token_cannot_be_shadowed_or_redirected() {
    let (_dir, k) = setup("k10-root-token");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let u = s.open_directory(None, &mut fds, b"user/.").unwrap();
    let o = s.open_directory(None, &mut fds, b"operator/.").unwrap();
    assert_ne!(u.0, o.0);
    s.close(&mut fds, u).unwrap();
    s.close(&mut fds, o).unwrap();
    // An explicit caller root must agree with the path token; it cannot move it.
    assert!(s.exists(Some(RootKind::Operator), b"user/x").is_err());
    assert!(s.exists(Some(RootKind::User), b"operator/x").is_err());
}

#[test]
fn cross_root_rename_cannot_move_private_data() {
    let (dir, k) = setup("k11-cross-root");
    let s = k.fs();
    std::fs::write(dir.join("operator/private"), b"private").unwrap();
    assert!(
        s.rename(None, b"operator/private", b"user/private")
            .is_err()
    );
    assert_eq!(
        std::fs::read(dir.join("operator/private")).unwrap(),
        b"private"
    );
    assert!(!dir.join("user/private").exists());
}
