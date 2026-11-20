use super::fixture::Fixture;
const GUEST: &str = r#"(module
(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))
(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
(import "skrynia" "try_mutation_lock"(func $a(result i32)))
(import "skrynia" "release_mutation_lock"(func $u(result i32)))
(memory(export "memory") 1)(data(i32.const 16)"OK")
(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)
  (if (i32.eqz(call $a))(then(return(i32.const -10))))
  (drop(call $r(local.get $i)(i32.const 0)(i32.const 1)))
  (if (i32.eq(i32.load8_u(i32.const 0))(i32.const 84))(then(unreachable)))
  (drop(call $u))
  (drop(call $w(local.get $o)(i32.const 16)(i32.const 2)))
  (i32.const 0)))"#;
#[test]
fn child_trap_does_not_strand_scoped_mutation_lock() {
    let f = Fixture::new(GUEST);
    assert!(f.start(false).is_err());
    let kernel = f.start(true).unwrap();
    assert!(kernel.invoke(b"T").is_err());
    assert_eq!(kernel.invoke(b"S").unwrap(), b"OK");
}
