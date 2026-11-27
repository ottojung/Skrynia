use super::case::check;
#[test]
fn startup_rejects_forbidden_or_mistyped_imports() {
    assert!(!check(
        "wasi",
        r#"(module
      (import "wasi_snapshot_preview1" "fd_write" (func))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
    assert!(!check(
        "bad-read",
        r#"(module
      (import "skrynia_v3_1" "read" (func(param i64)(result i32)))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
    assert!(!check(
        "extra",
        r#"(module
      (import "skrynia_v3_1" "extra" (func))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
    assert!(
        !check(
            "old-namespace",
            r#"(module
      (import "skrynia" "read1" (func (param i32) (result i32)))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
        ),
        "old namespace must be rejected"
    );
}
