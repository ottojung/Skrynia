use libskrynia::{EquivalentCall, Syscalls};
#[test]
fn invalid_imports_fail_at_startup() {
    assert!(super::super::kernel(super::super::ROOT, Syscalls::empty()).is_err());
    assert!(EquivalentCall::new("../bad", Ok).is_err());
    assert!(
        Syscalls::optimized([
            EquivalentCall::new("double", Ok).unwrap(),
            EquivalentCall::new("double", Ok).unwrap(),
        ])
        .is_err()
    );
    let bad = super::super::ROOT
        .replace("(param i32) (result i32)))", "(param i32) (result i64)))")
        .replace(
            "(call $double (i32.const -1))",
            "(i32.wrap_i64 (call $double (i32.const -1)))",
        );
    let calls = Syscalls::optimized([EquivalentCall::new("double", Ok).unwrap()]).unwrap();
    assert!(super::super::kernel(&bad, calls).is_err());
}

#[test]
fn equivalent_failure_traps() {
    let calls =
        Syscalls::optimized([
            EquivalentCall::new("double", |_| Err("endpoint failure".into())).unwrap(),
        ])
        .unwrap();
    assert!(
        super::super::kernel(super::super::ROOT, calls)
            .unwrap()
            .invoke(b"")
            .is_err()
    );
}
