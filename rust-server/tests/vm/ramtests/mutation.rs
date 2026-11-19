use skrynia_server::boot;
use std::fs;

pub(super) fn kernel() -> boot::Kernel {
    let base = std::env::temp_dir().join(format!(
        "skrynia-v3-mutation-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("data")).unwrap();
    let source = r#"(module
(import "skrynia" "read" (func $read (param i32 i32 i32) (result i32)))
(import "skrynia" "write" (func $write (param i32 i32 i32) (result i32)))
(import "skrynia" "try_mutation_lock" (func $acquire (result i32)))
(import "skrynia" "release_mutation_lock" (func $release (result i32)))
(memory (export "memory") 1)
(func (export "skrynia_main") (param i32 i32) (result i32)
  (drop (call $read (local.get 0) (i32.const 0) (i32.const 1)))
  (if (i32.eq (i32.load8_u (i32.const 0)) (i32.const 77))
    (then (return (call $release))))
  (if (i32.eq (call $acquire) (i32.const 0)) (then (return (i32.const -10))))
  (if (i32.eq (i32.load8_u (i32.const 0)) (i32.const 84)) (then (unreachable)))
  (if (i32.eq (i32.load8_u (i32.const 0)) (i32.const 70))
    (then (loop $spin (br $spin))))
  (drop (call $write (local.get 1) (i32.const 0) (i32.const 1)))
  (call $release)))"#;
    fs::write(base.join("root.wasm"), wat::parse_str(source).unwrap()).unwrap();
    boot::Kernel::start(&base.join("root.wasm"), &base.join("data")).unwrap()
}

#[test]
fn request_owned_lock_survives_traps_and_does_not_unlock_other_vm() {
    let k = kernel();
    assert!(
        k.invoke(b"T").is_err(),
        "request traps while holding mutation lock"
    );
    assert_eq!(k.invoke(b"X").unwrap(), b"X");
    assert!(
        k.invoke(b"F").is_err(),
        "fuel is exhausted inside kernel-owned lock"
    );
    assert_eq!(k.invoke(b"Y").unwrap(), b"Y");
    assert_eq!(
        k.invoke(b"M"),
        Err(-4),
        "non-owner must never release another owner"
    );
    assert_eq!(k.invoke(b"Z").unwrap(), b"Z");
}
