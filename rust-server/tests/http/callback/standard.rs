#[test]
fn nested_eval_callback() {
    let root = r#"(module
(import "skrynia" "read" (func $r(param i32 i32 i32)(result i32)))
(import "skrynia" "eval" (func $e(param i32 i32 i32 i32 i32)(result i32)))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 3)
(func $cb(param i32 i32 i32 i32)(result i32)
  (i32.store8(local.get 2)(i32.const 75))(i32.const 1))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
  (local $n i32)
  (local.set $n(call $r(local.get $in)(i32.const 32768)(i32.const 32768)))
  (if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))
  (call $e(i32.const 32768)(local.get $n)(local.get $in)(local.get $out)(i32.const 0))))"#;
    let child = r#"(module
(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))
(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
  (local $n i32)
  (local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 8)))
  (if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))
  (drop(call $w(local.get $out)(i32.const 64)(local.get $n)))
  (i32.const 0)))"#;
    let dir = std::env::temp_dir().join("skrynia-v3-nested");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(k.invoke(&wat::parse_str(child).unwrap()).unwrap(), b"K");
}
