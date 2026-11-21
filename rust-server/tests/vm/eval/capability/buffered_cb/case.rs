use super::util::{hex, run};

#[test]
fn buffered_callback_uses_bounded_provider_authority() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)
(data(i32.const 0)"allowed")(data(i32.const 16)"secret")
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $a i32)(local $b i32)
  (local.set $a(call $i(i32.const 0)(i32.const 7)(i32.const 64)(i32.const 8)))
  (if(i32.ne(local.get $a)(i32.const 0))(then(return(i32.const -11))))
  (local.set $b(call $i(i32.const 16)(i32.const 6)(i32.const 64)(i32.const 8)))
  (if(i32.ge_s(local.get $b)(i32.const 0))(then(return(i32.const -12))))
  (i32.store8(i32.const 80)(i32.const 79))(i32.store8(i32.const 81)(i32.const 75))
  (drop(call $w(local.get $out)(i32.const 80)(i32.const 2)))(i32.const 0)))"#).unwrap();

    let p1 = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)
(data(i32.const 0) "{p2h}")(data(i32.const 1024) "allowed")
(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)
  (call $ds(i32.const 0)(local.get $rp)(local.get $rl)(i32.const 127)))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)
  (drop(call $ds(i32.const 0)(i32.const 1024)(i32.const 7)(i32.const 127)))
  (local.set $n(call $e(i32.const 0)(i32.const {p2l})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 16)(i32.const 0)))
  (if(i32.lt_s(local.get $n)(i32.const 0))(then(i32.store(i32.const 4000)(local.get $n))(drop(call $w(local.get $out)(i32.const 4000)(i32.const 4)))(return(i32.const 0))))
  (drop(call $w(local.get $out)(i32.const 3000)(local.get $n)))(i32.const 0)))"#,
        p2h = hex(&p2),
        p2l = p2.len()
    );
    let p1b = wat::parse_str(&p1).unwrap();

    let root = format!(
        r#"(module(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{p1h}")
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
  (drop(call $ds(i32.const 0)(i32.const 0)(i32.const 0)(i32.const 127)))
  (call $ed(i32.const 0)(i32.const {p1l})(local.get $in)(local.get $out)(i32.const -1))))"#,
        p1h = hex(&p1b),
        p1l = p1b.len()
    );
    assert_eq!(run("buf-callback-auth", &root).unwrap(), b"OK");
}
