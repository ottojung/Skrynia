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
      (import "skrynia" "read" (func(param i64)(result i32)))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
    assert!(!check(
        "extra",
        r#"(module
      (import "skrynia" "extra" (func))
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
}
