use libskrynia::boot::Kernel;
use std::fs;

#[test]
fn oversized_eval_code_size_is_rejected_before_ram_read() {
    let root = r#"(module
(import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(drop(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $len))
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
  (local $r i64)
  (local.set $r(call $e(i64.const 0)(i64.const 9437184)(i32.const -1)(i32.const 0)(i32.const 0)))
  (i32.store8(i32.const 100)(i64.eq(local.get $r)(i64.const -6)))
  (drop(call $w(local.get $out)(i32.const 100)(i32.const 1)))
  (i32.const 0)))"#;
    let dir = std::env::temp_dir().join("k2-eval-limit-size");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    assert_eq!(k.invoke(b"").unwrap(), vec![1]);
    let _ = fs::remove_dir_all(&dir);
}
