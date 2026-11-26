use super::support::setup;
use libskrynia::boot::vm::cap::{FsAuth, FsCaps, FsOp, FsRoot};
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
        assert!(s.open_directory(None, &mut fds, p, 0).is_err(), "{p:?}");
    }
}

#[test]
fn root_token_cannot_be_shadowed_or_redirected() {
    let (_dir, k) = setup("k10-root-token");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let u = s.open_directory(None, &mut fds, b"user/.", 0).unwrap();
    let o = s.open_directory(None, &mut fds, b"operator/.", 0).unwrap();
    assert_ne!(u.0, o.0);
    s.close(&mut fds, u).unwrap();
    s.close(&mut fds, o).unwrap();
    // An explicit caller root must agree with the path token; it cannot move it.
    assert!(s.exists(Some(RootKind::Operator), b"user/x").is_err());
    assert!(s.exists(Some(RootKind::User), b"operator/x").is_err());
}

#[test]
fn scoped_caps_can_only_restrict_the_selected_namespace() {
    let caps = FsAuth::Scoped(vec![FsCaps {
        root: FsRoot::User,
        prefix: b"".to_vec(),
        ops: 1 << FsOp::Exists as u32,
    }]);
    assert!(caps.allows(FsRoot::User, FsOp::Exists, b"k.bin"));
    assert!(!caps.allows(FsRoot::Operator, FsOp::Exists, b"k.bin"));
    assert!(!caps.allows(FsRoot::User, FsOp::Delete, b"k.bin"));
    assert!(!FsAuth::Empty.allows(FsRoot::User, FsOp::Exists, b"k.bin"));
    assert!(FsAuth::Full.allows(FsRoot::Operator, FsOp::Exists, b"k.bin"));
}
