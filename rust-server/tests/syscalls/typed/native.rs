use libskrynia::{EquivalentCall, Syscalls};
use wasmtime::Val;

pub(super) fn mix() -> EquivalentCall {
    EquivalentCall::new("mix", "ili", "li", |args| {
        let a = args[0].unwrap_i32() as i64;
        let b = args[1].unwrap_i64();
        let c = args[2].unwrap_i32();
        Ok(vec![
            Val::I64(b.wrapping_add(a)),
            Val::I32(c.wrapping_mul(3)),
        ])
    })
    .unwrap()
}

#[test]
fn mixed_widths_and_multivalue_match_typed_signature() {
    let wat = super::root(super::MIX_IMPORT, super::MIX_BODY);
    let calls = Syscalls::optimized([mix()]).unwrap();
    let k = super::super::kernel(&wat, calls).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), super::MIX_BYTES);
}

#[test]
fn zero_args_and_zero_results() {
    let wat = super::root(
        r#"(import "syscall" "tick" (func $tick))"#,
        "(call $tick) (i32.store8 (i32.const 0) (i32.const 84)) (drop (call $w (local.get $out) (i32.const 0) (i32.const 1)))",
    );
    let tick = EquivalentCall::new("tick", "", "", |_| Ok(vec![])).unwrap();
    let k = super::super::kernel(&wat, Syscalls::optimized([tick]).unwrap()).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), b"T");
}

#[test]
fn zero_args_with_one_i64_result() {
    let wat = super::root(
        r#"(import "syscall" "one" (func $one (result i64)))"#,
        "(i64.store (i32.const 0) (call $one)) (drop (call $w (local.get $out) (i32.const 0) (i32.const 8)))",
    );
    let one = EquivalentCall::new("one", "", "l", |_| Ok(vec![Val::I64(9)])).unwrap();
    let k = super::super::kernel(&wat, Syscalls::optimized([one]).unwrap()).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), 9i64.to_le_bytes());
}
