use libskrynia::{EquivalentCall, Syscalls};

fn double() -> EquivalentCall {
    EquivalentCall::new("double", "i", "i", |args| Ok(args.to_vec())).unwrap()
}

#[test]
fn invalid_imports_fail_at_startup() {
    assert!(super::super::super::kernel(super::super::super::ROOT, Syscalls::empty()).is_err());
    assert!(EquivalentCall::new("../bad", "", "", |_| Ok(vec![])).is_err());
    assert!(Syscalls::optimized([double(), double()]).is_err());
    let bad = super::super::super::ROOT
        .replace("(param i32) (result i32)))", "(param i32) (result i64)))")
        .replace(
            "(call $double (i32.const -1))",
            "(i32.wrap_i64 (call $double (i32.const -1)))",
        );
    assert!(super::super::super::kernel(&bad, Syscalls::optimized([double()]).unwrap()).is_err());
    let wrong =
        Syscalls::optimized([
            EquivalentCall::new("double", "l", "i", |args| Ok(args.to_vec())).unwrap(),
        ])
        .unwrap();
    assert!(super::super::super::kernel(super::super::super::ROOT, wrong).is_err());
}
