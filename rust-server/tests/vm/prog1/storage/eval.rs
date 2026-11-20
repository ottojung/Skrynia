use super::fixture::Fixture;
#[test]
fn scoped_program1_can_evaluate_independent_unprivileged_user_wasm() {
    let guest = wat::parse_str(
        r#"(module
      (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)
        (i32.store8(local.get 2)(i32.const 79))
        (i32.store8(i32.add(local.get 2)(i32.const 1))(i32.const 75))
        (i32.const 2)))"#,
    )
    .unwrap();
    let bytes = guest
        .iter()
        .map(|b| format!("\\{b:02x}"))
        .collect::<String>();
    let program = format!(
        r#"(module
      (import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))
      (import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
      (memory(export "memory") 2)
      (data(i32.const 0)"{bytes}")
      (func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)
        (local $n i32)
        (local.set $n(call $e(i32.const 0)(i32.const {length})
           (i32.const 0)(i32.const 0)(i32.const -1)
           (i32.const 1024)(i32.const 64)))
        (if (i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))
        (drop(call $w(local.get $o)(i32.const 1024)(local.get $n)))
        (i32.const 0)))"#,
        length = guest.len()
    );
    let f = Fixture::new(&program);
    assert!(f.start(false).is_err(), "default guest cannot eval");
    assert_eq!(f.start(true).unwrap().invoke(b"").unwrap(), b"OK");
}
