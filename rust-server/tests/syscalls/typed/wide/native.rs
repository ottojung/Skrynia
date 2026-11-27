use libskrynia::{EquivalentCall, Syscalls};
use wasmtime::Val;

#[test]
fn three_hundred_params_and_one_result_are_accepted() {
    let call = EquivalentCall::new("wide", &"i".repeat(super::N), "i", |args| {
        let sum: i32 = args.iter().map(|v| v.unwrap_i32()).sum();
        Ok(vec![Val::I32(sum)])
    })
    .unwrap();
    let import = format!(
        r#"(import "syscall" "wide" (func $wide {} (result i32)))"#,
        super::params(super::N)
    );
    let body = format!(
        "(i32.store (i32.const 0) (call $wide {})) (drop (call $w (local.get $out) (i32.const 0) (i32.const 4)))",
        super::args(super::N)
    );
    let k = super::super::kernel(
        &super::super::root(&import, &body),
        Syscalls::optimized([call]).unwrap(),
    )
    .unwrap();
    let expected = (super::N * (super::N - 1) / 2) as i32;
    assert_eq!(k.invoke(b"").unwrap(), expected.to_le_bytes());
}

#[test]
fn three_hundred_results_roundtrip_through_typed_ffi() {
    let call = EquivalentCall::new("wide", "", &"i".repeat(super::N), |_| {
        Ok((0..super::N).map(|i| Val::I32(i as i32)).collect())
    })
    .unwrap();
    let import = format!(
        r#"(import "syscall" "wide" (func $wide {}))"#,
        super::results(super::N)
    );
    let body = format!(
        "(call $wide) {} (drop (call $w (local.get $out) (i32.const 0) (i32.const {})))",
        super::stores(super::N),
        super::N * 4
    );
    let module = super::super::root_with(&super::put(), &import, &body);
    let k = super::super::kernel(&module, Syscalls::optimized([call]).unwrap()).unwrap();
    let values: Vec<u64> = (0..super::N as u64).collect();
    assert_eq!(k.invoke(b"").unwrap(), super::bytes(&values));
}
