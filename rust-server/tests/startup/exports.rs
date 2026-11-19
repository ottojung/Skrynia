use super::case::check;
#[test]
fn startup_checks_exports_without_invoking_root() {
    let good = include_str!("../../test-fixtures/root_stub.wat");
    assert!(check("good", good));
    assert!(!check(
        "no-memory",
        r#"(module
      (func(export "skrynia_main")(param i32 i32)(result i32) i32.const 0))"#
    ));
    assert!(!check(
        "bad-main",
        r#"(module(memory(export "memory") 1)
      (func(export "skrynia_main")(param i32)(result i32) i32.const 0))"#
    ));
    assert!(!check(
        "wrong-return",
        r#"(module(memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i64) i64.const 0))"#
    ));
}
