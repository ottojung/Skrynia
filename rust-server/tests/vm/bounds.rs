use skrynia_server::boot::Kernel;
use std::fs;

#[test]
fn oversized_eval_module_fails_before_reading_guest_memory() {
    let root = r#"(module
(import "skrynia" "write" (func $w(param i32 i32 i32)(result i32)))
(import "skrynia" "eval" (func $e(param i32 i32 i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
  (local $r i32)
  (local.set $r(call $e(i32.const 0)(i32.const 9437184)(local.get $in)(local.get $out)(i32.const -1)))
  (i32.store8(i32.const 100)(i32.eq(local.get $r)(i32.const -10)))
  (drop(call $w(local.get $out)(i32.const 100)(i32.const 1)))
  (i32.const 0)))"#;
    let dir = std::env::temp_dir().join("v3-eval-limit-fd");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(k.invoke(b"").unwrap(), vec![1]);
}
