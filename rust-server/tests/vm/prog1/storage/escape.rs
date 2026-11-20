use super::fixture::Fixture;
use std::os::unix::fs::symlink;
const GUEST: &str = r#"(module
(import "skrynia" "open_disk"(func $o(param i32 i32 i32)(result i32)))
(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(data(i32.const 0)"\01\01")
(data(i32.const 32)"../escape")
(data(i32.const 64)"link/escape")
(func(export "skrynia_main")(param $i i32)(param $out i32)(result i32)
  (local $a i32)(local $b i32)
  (local.set $a(call $o(i32.const 32)(i32.const 9)(i32.const 6)))
  (local.set $b(call $o(i32.const 64)(i32.const 11)(i32.const 6)))
  (if (i32.ge_s(local.get $a)(i32.const 0))(then(return(i32.const -9))))
  (if (i32.ge_s(local.get $b)(i32.const 0))(then(return(i32.const -9))))
  (drop(call $w(local.get $out)(i32.const 0)(i32.const 2)))
  (i32.const 0)))"#;

#[test]
fn traversal_and_symlink_escapes_are_denied() {
    let f = Fixture::new(GUEST);
    symlink(f.path.join("outside"), f.path.join("service/link")).unwrap();
    assert_eq!(f.start(true).unwrap().invoke(b"escape").unwrap(), [1, 1]);
    assert!(!f.path.join("escape").exists());
    assert!(!f.path.join("outside/escape").exists());
}
