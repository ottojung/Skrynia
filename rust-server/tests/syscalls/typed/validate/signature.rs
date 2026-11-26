use libskrynia::{EquivalentCall, Syscalls};
use wasmtime::Val;

#[test]
fn malformed_signature_is_rejected_before_invocation() {
    let wat = super::super::root(super::super::MIX_IMPORT, super::super::MIX_BODY);
    assert!(super::super::super::kernel(&wat, Syscalls::empty()).is_err());
    let wrong_args = Syscalls::optimized([EquivalentCall::new("mix", "il", "li", |_| {
        Ok(vec![Val::I64(0), Val::I32(0)])
    })
    .unwrap()])
    .unwrap();
    assert!(super::super::super::kernel(&wat, wrong_args).is_err());
    for results in [vec![Val::I64(0)], vec![Val::I32(0), Val::I32(0)]] {
        let calls = Syscalls::optimized([EquivalentCall::new("mix", "ili", "li", move |_| {
            Ok(results.clone())
        })
        .unwrap()])
        .unwrap();
        assert!(
            super::super::super::kernel(&wat, calls)
                .unwrap()
                .invoke(b"")
                .is_err()
        );
    }
}
