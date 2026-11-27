use super::support::start_at;
use std::fs;
const GUEST: &str = r#"(module
 (import "skrynia_v3_1" "open_disk" (func $od (param i64 i64 i32) (result i32)))
 (import "skrynia_v3_1" "read1" (func $ri (param i32) (result i32)))
 (import "skrynia_v3_1" "close" (func $cl (param i32) (result i32)))
 (import "skrynia_v3_1" "write1" (func $wo (param i32 i32) (result i32)))
 (memory (export "memory") 1)
 (func $r (param $fd i32)(param $d i32)(param $l i32)(result i32)(local $i i32)(local $b i32)
  (block $x(loop $lp(br_if $x(i32.ge_u(local.get $i)(local.get $l)))
   (local.set $b(call $ri(local.get $fd)))
   (if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $x)))
   (i32.store8(i32.add(local.get $d)(local.get $i))(local.get $b))
   (local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $op i32)(local $v i32)
  (local.set $op (call $ri (local.get $in)))
  (if (i32.eq (local.get $op) (i32.const 66))
   (then (local.set $v (call $cl (i32.const 131073))))
   (else (drop (call $r (local.get $in) (i32.const 0) (i32.const 11)))
    (local.set $v (call $od (i64.const 0) (i64.const 11) (i32.const 3)))))
  (drop (call $wo (local.get $out) (i32.and (local.get $v) (i32.const 255)))) (i32.const 0)))"#;
#[test]
fn missing_path_is_distinct_from_bad_descriptor_and_io() {
    let dir = std::env::temp_dir().join(format!("k10-errno-{}", rand::random::<u64>()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let k = start_at(&dir, GUEST);
    assert_eq!(
        k.invoke(b"Muser/absent").unwrap(),
        vec![0xf8],
        "ENOENT is -8"
    );
    assert_eq!(k.invoke(b"B").unwrap(), vec![0xfe], "EBADF is -2");
    for p in [b"Mabsent11111".as_slice(), b"Muser/../abc", b"Muser//abcde"] {
        assert_eq!(k.invoke(p).unwrap(), vec![0xfd], "invalid path is -3");
    }
    fs::write(dir.join("user/absent"), b"x").unwrap();
    assert_ne!(
        k.invoke(b"Muser/absent").unwrap(),
        vec![0xf8],
        "present path is not absent"
    );
    fs::remove_dir_all(&dir).unwrap();
}
