use super::stream::run;
#[test]
fn oversized_eval_module_fails_before_reading_guest_memory() {
    let root = r#"(module
(import "skrynia" "eval"
  (func $e (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
(memory (export "memory") 1)
(func(export "skrynia_main") (param i32 i32 i32 i32) (result i32)
  (i32.store8 (local.get 2)
    (i32.eq (call $e (i32.const 0) (i32.const 9437184)
      (i32.const 0) (i32.const 0) (i32.const -1)
      (local.get 2) (i32.const 1)) (i32.const -10)))
  (i32.const 1)))"#;
    assert_eq!(run("v3-eval-limit", root, b"").unwrap(), &[1]);
}
#[test]
fn bounded_compile_returns_promptly() {
    let mut wat = String::from("(module");
    for _ in 0..4000 {
        wat.push_str("(func(param i32)(result i32)(local.get 0))");
    }
    wat.push(')');
    let wasm = wat::parse_str(&wat).unwrap();
    let eng = wasmtime::Engine::default();
    let t = std::time::Instant::now();
    let r = skrynia_server::boot::vm::compile::bounded(&eng, &wasm);
    assert!(t.elapsed() < std::time::Duration::from_millis(800));
    assert!(r.is_ok() || matches!(r, Err("limit") | Err("invalid")));
}
