use super::basic::{hex, run};

#[test]
fn buffered_eval_fuel_exhaustion_clean_failure() {
    let spin = wat::parse_str(
        r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(loop br 0)(i32.const 0)))"#,
    )
    .unwrap();
    let hs = hex(&spin);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{hs}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {ls})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const -1)))(i32.store(i32.const 1024)(local.get $n))(drop(call $w(local.get $out)(i32.const 1024)(i32.const 4)))(i32.const 0)))"#,
        hs = hs,
        ls = spin.len()
    );
    let res = run("buf-fuel-clean", &root).unwrap();
    assert_eq!(res.len(), 4);
    let code = i32::from_le_bytes(res.try_into().unwrap());
    assert!(code < 0, "expected fuel/trap error, got {code}");
}

#[test]
fn buffered_eval_reentrant_callback_trap_revive() {
    let p2 = wat::parse_str(
        r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 1)(i32.const 64)(i32.const 8)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(drop(call $w(local.get $out)(i32.const 64)(i32.const 1)))(i32.const 0)))"#,
    )
    .unwrap();
    let p1 = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 1024) "{}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $n i32)(local.set $n(call $i(local.get $rp)(local.get $rl)(local.get $dp)(local.get $dc)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.get $n))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 1024)(i32.const {})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const 0)))(if(i32.ge_s(local.get $n)(i32.const 0))(then(return(i32.const -5))))(local.set $n(call $e(i32.const 1024)(i32.const {})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const 0)))(if(i32.ne(local.get $n)(i32.const 1))(then(return(local.get $n))))(drop(call $w(local.get $out)(i32.const 3000)(i32.const 1)))(i32.const 0)))"#,
        hex(&p2),
        p2.len(),
        p2.len()
    );
    let p1b = wat::parse_str(&p1).unwrap();
    let root = format!(
        r#"(module(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{}")(global $armed (mut i32)(i32.const 0))(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(if(i32.eq(global.get $armed)(i32.const 0))(then(global.set $armed(i32.const 1))(unreachable)))(i32.store8(local.get $dp)(i32.const 77))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {})(local.get $in)(local.get $out)(i32.const 0))))"#,
        hex(&p1b),
        p1b.len()
    );
    assert_eq!(run("buf-reentrant-trap", &root).unwrap(), vec![77]);
}
