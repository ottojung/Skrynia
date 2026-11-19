#[test]
fn nested_eval_callback() {
    let root = r#"(module
(import "skrynia" "read" (func $r(param i32 i32 i32)(result i32)))
(import "skrynia" "write" (func $w(param i32 i32 i32)(result i32)))
(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 3)
(func $cb(param i32 i32 i32 i32)(result i32)
  (i32.store8(local.get 2)(i32.const 75))(i32.const 1))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $input i32)(param $output i32)(result i32)
  (local $n i32)(local $result i32)
  (local.set $n(call $r(local.get $input)(i32.const 32768)(i32.const 32768)))
  (local.set $result(call $e(i32.const 32768)(local.get $n)
    (i32.const 65536)(i32.const 0)(i32.const 0)(i32.const 70000)(i32.const 1024)))
  (if(i32.lt_s(local.get $result)(i32.const 0))(then(return(local.get $result))))
  (if(i32.ne(call $w(local.get $output)(i32.const 70000)(local.get $result))(local.get $result))
    (then(return(i32.const -5))))
  (i32.const 0)))"#;
    let child = r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(call $i(i32.const 0)(i32.const 0)(local.get 2)(local.get 3))))"#;
    let dir = std::env::temp_dir().join("skrynia-v3-nested");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(k.invoke(&wat::parse_str(child).unwrap()).unwrap(), b"K");
}
