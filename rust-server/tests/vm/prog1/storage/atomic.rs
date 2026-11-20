use super::fixture::Fixture;
use std::fs;
const GUEST: &str = r#"(module
(import "skrynia" "mkdir"(func $mkdir(param i32 i32 i32)(result i32)))
(import "skrynia" "open_disk"(func $open(param i32 i32 i32)(result i32)))
(import "skrynia" "write"(func $write(param i32 i32 i32)(result i32)))
(import "skrynia" "sync"(func $sync(param i32)(result i32)))
(import "skrynia" "close"(func $close(param i32)(result i32)))
(import "skrynia" "rename"(func $rename(param i32 i32 i32 i32)(result i32)))
(import "skrynia" "open_directory"(func $dir(param i32 i32)(result i32)))
(import "skrynia" "try_mutation_lock"(func $acq(result i32)))
(import "skrynia" "release_mutation_lock"(func $rel(result i32)))
(memory(export "memory") 1)
(data(i32.const 0)"app")
(data(i32.const 16)"app/pending")
(data(i32.const 32)"app/active")
(data(i32.const 64)"v1")
(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)
  (local $fd i32)(local $dir i32)
  (if (i32.eqz(call $acq))(then(return(i32.const -10))))
  (drop(call $mkdir(i32.const 0)(i32.const 3)(i32.const 4)))
  (local.set $fd(call $open(i32.const 16)(i32.const 11)(i32.const 14)))
  (if (i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))
  (drop(call $write(local.get $fd)(i32.const 64)(i32.const 2)))
  (drop(call $sync(local.get $fd)))
  (drop(call $close(local.get $fd)))
  (drop(call $rename(i32.const 16)(i32.const 11)(i32.const 32)(i32.const 10)))
  (local.set $dir(call $dir(i32.const 0)(i32.const 3)))
  (drop(call $sync(local.get $dir)))
  (drop(call $close(local.get $dir)))
  (drop(call $rel))
  (drop(call $write(local.get $o)(i32.const 64)(i32.const 2)))
  (i32.const 0)))"#;
#[test]
fn only_scoped_child_can_atomically_publish_a_data_pointer() {
    let f = Fixture::new(GUEST);
    assert!(f.start(false).is_err());
    let kernel = f.start(true).unwrap();
    assert_eq!(kernel.invoke(b"commit").unwrap(), b"v1");
    assert_eq!(fs::read(f.path.join("service/app/active")).unwrap(), b"v1");
    assert!(!f.path.join("service/app/pending").exists());
    assert!(!f.path.join("kernel/app/active").exists());
}
