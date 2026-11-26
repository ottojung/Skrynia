use super::support::start_at;
use std::fs;

const GUEST: &str = r#"(module
 (import "skrynia" "open_disk" (func $od (param i64 i64 i32) (result i32)))
 (import "skrynia" "read1" (func $ri (param i32) (result i32)))
 (import "skrynia" "close" (func $cl (param i32) (result i32)))
 (import "skrynia" "write1" (func $wo (param i32 i32) (result i32)))
 (memory (export "memory") 1)
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $op i32)(local $v i32)
  (local.set $op (call $ri (local.get $in)))
  (if (i32.eq (local.get $op) (i32.const 66))
   (then (local.set $v (call $cl (i32.const 131073))))
   (else (local.set $v (call $od (i64.const 0) (i64.const 11) (i32.const 3)))))
  (drop (call $wo (local.get $out) (i32.and (local.get $v) (i32.const 255))))
  (i32.const 0)))"#;

#[test]
fn missing_path_is_distinct_from_bad_descriptor_and_io() {
    let dir = std::env::temp_dir().join(format!("k10-errno-{}", rand::random::<u64>()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let k = start_at(&dir, GUEST);
    k.ram().truncate(64).unwrap();
    k.ram().write(0, b"user/absent").unwrap();
    assert_eq!(k.invoke(b"M").unwrap(), vec![0xf8], "ENOENT is -8");
    assert_eq!(k.invoke(b"B").unwrap(), vec![0xfe], "EBADF is -2");
    for p in [
        b"absent11111".as_slice(),
        b"user/../operator/ab",
        b"user//abcde",
    ] {
        k.ram().write(0, p).unwrap();
        assert_eq!(k.invoke(b"M").unwrap(), vec![0xfd], "invalid path is -3");
    }
    fs::write(dir.join("user/absent"), b"x").unwrap();
    k.ram().write(0, b"user/absent").unwrap();
    assert_ne!(
        k.invoke(b"M").unwrap(),
        vec![0xf8],
        "present path is not absent"
    );
    fs::remove_dir_all(&dir).unwrap();
}
