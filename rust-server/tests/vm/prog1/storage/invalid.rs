use super::fixture::Fixture;
use std::os::unix::fs::symlink;
const READ_ONLY: &str = r#"(module
  (memory(export "memory") 1)
  (func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
const RAM: &str = r#"(module
  (import "skrynia" "open_ram"(func $r(result i32)))
  (memory(export "memory") 1)
  (func(export "skrynia_main")(param i32 i32)(result i32)(call $r)))"#;

#[test]
fn scoped_guest_cannot_import_unrelated_global_ram() {
    assert!(Fixture::new(RAM).start(true).is_err());
}
#[test]
fn operator_scope_symlink_is_rejected_before_requests() {
    let f = Fixture::new(READ_ONLY);
    std::fs::remove_dir(f.path.join("service")).unwrap();
    symlink(f.path.join("outside"), f.path.join("service")).unwrap();
    assert!(f.start(true).is_err());
}
#[test]
fn storage_scope_without_a_child_program_is_invalid() {
    let f = Fixture::new(READ_ONLY);
    let bad = skrynia_server::boot::Kernel::start_with_scope(
        &f.path.join("root.wasm"),
        &f.path.join("kernel"),
        None,
        Some(&f.path.join("service")),
    );
    assert!(bad.is_err());
}
