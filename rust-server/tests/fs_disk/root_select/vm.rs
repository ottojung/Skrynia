use super::support::start_at;
use std::fs;
const GUEST: &str = r#"(module
 (import "skrynia_v3_1" "open_disk" (func $od (param i64 i64 i32) (result i32)))
 (import "skrynia_v3_1" "write1" (func $w1 (param i32 i32) (result i32)))
 (import "skrynia_v3_1" "write1" (func $wo (param i32 i32) (result i32)))
 (import "skrynia_v3_1" "close" (func $cl (param i32) (result i32)))
 (import "skrynia_v3_1" "read1" (func $ri (param i32) (result i32)))
 (memory (export "memory") 1)
 (func $r (param $fd i32)(param $d i32)(param $l i32)(result i32)(local $i i32)(local $b i32)
  (block $x(loop $lp(br_if $x(i32.ge_u(local.get $i)(local.get $l)))
   (local.set $b(call $ri(local.get $fd)))
   (if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $x)))
   (i32.store8(i32.add(local.get $d)(local.get $i))(local.get $b))
   (local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)(local $b i32)(local $op i32)
  (local.set $op (call $ri (local.get $in)))
  (drop (call $r (local.get $in) (i32.const 0) (i32.const 11)))
  (if (i32.eq (local.get $op) (i32.const 87))
   (then (local.set $fd (call $od (i64.const 0) (i64.const 11) (i32.const 7)))
    (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
    (drop (call $w1 (local.get $fd) (i32.const 90))) (drop (call $cl (local.get $fd))) (return (i32.const 0))))
  (local.set $fd (call $od (i64.const 0) (i64.const 11) (i32.const 3)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
  (local.set $b (call $ri (local.get $fd))) (drop (call $cl (local.get $fd)))
  (drop (call $wo (local.get $out) (local.get $b))) (i32.const 0)))"#;
#[test]
fn vm_persistence_is_genuine_and_root_confined() {
    let dir = std::env::temp_dir().join(format!("k10-vm-{}", rand::random::<u64>()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("root.wasm"), wat::parse_str(GUEST).unwrap()).unwrap();
    let k = start_at(&dir, GUEST);
    assert_eq!(k.invoke(b"Wuser/abcdef").unwrap(), Vec::<u8>::new());
    assert_eq!(k.invoke(b"Ruser/abcdef").unwrap(), vec![90]);
    assert!(k.invoke(b"Roperator/ab").is_err());
    drop(k);
    let k2 = start_at(&dir, GUEST);
    assert_eq!(k2.invoke(b"Ruser/abcdef").unwrap(), vec![90]);
    fs::remove_dir_all(&dir).unwrap();
}
