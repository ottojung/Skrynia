use libskrynia::{EquivalentCall, Syscalls};
use wasmtime::Val;

#[test]
fn equivalent_failure_and_bad_result_types_trap() {
    let calls = Syscalls::optimized([EquivalentCall::new("double", "i", "i", |_| {
        Err("endpoint failure".into())
    })
    .unwrap()])
    .unwrap();
    assert!(
        super::super::super::kernel(super::super::super::ROOT, calls)
            .unwrap()
            .invoke(b"")
            .is_err()
    );
    let wrong =
        Syscalls::optimized([
            EquivalentCall::new("double", "i", "i", |_| Ok(vec![Val::I64(0)])).unwrap(),
        ])
        .unwrap();
    assert!(
        super::super::super::kernel(super::super::super::ROOT, wrong)
            .unwrap()
            .invoke(b"")
            .is_err()
    );
}
