use super::fixture::Fixture;
use std::fs;
const GUEST: &str = r#"(module
(import "skrynia" "open_disk"(func $o(param i32 i32 i32)(result i32)))
(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
(import "skrynia" "close"(func $c(param i32)(result i32)))
(memory(export "memory") 1)
(data(i32.const 0)"hello")
(data(i32.const 32)"record")
(func(export "skrynia_main")(param $i i32)(param $out i32)(result i32)
  (local $fd i32)
  (local.set $fd (call $o (i32.const 32)(i32.const 6)(i32.const 6)))
  (if (i32.lt_s(local.get $fd)(i32.const 0))
    (then(return(local.get $fd))))
  (drop(call $w(local.get $fd)(i32.const 0)(i32.const 5)))
  (drop(call $c(local.get $fd)))
  (drop(call $w(local.get $out)(i32.const 0)(i32.const 5)))
  (i32.const 0)))"#;

#[test]
fn scoped_storage_is_default_deny_and_never_aliases_root_data() {
    let f = Fixture::new(GUEST);
    assert!(
        f.start(false).is_err(),
        "storage imports require explicit operator scope"
    );
    let kernel = f.start(true).unwrap();
    assert_eq!(kernel.invoke(b"unused").unwrap(), b"hello");
    assert_eq!(fs::read(f.path.join("service/record")).unwrap(), b"hello");
    assert!(!f.path.join("kernel/record").exists());
    assert!(!f.path.join("outside/record").exists());
}
