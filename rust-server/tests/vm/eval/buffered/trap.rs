use super::basic::{hex, run};

#[test]
fn buffered_eval_child_trap_and_revive() {
    let bad = wat::parse_str(r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(unreachable)))"#).unwrap();
    let ok = wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "OK")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 2)))(i32.const 0)))"#).unwrap();
    let hb = hex(&bad);
    let ho = hex(&ok);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{hb}")(data(i32.const 512) "{ho}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {lb})(i32.const 2000)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const -1)))(if(i32.ge_s(local.get $n)(i32.const 0))(then(return(i32.const -5))))(local.set $n(call $e(i32.const 512)(i32.const {lo})(i32.const 2000)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 2))(then(return(local.get $n))))(if(i32.ne(i32.load16_u(i32.const 3000))(i32.const 0x4b4f))(then(return(i32.const -6))))(i32.const 0)))"#,
        hb = hb,
        ho = ho,
        lb = bad.len(),
        lo = ok.len()
    );
    assert_eq!(run("buf-trap-revive", &root).unwrap(), Vec::<u8>::new());
}

#[test]
fn buffered_eval_callback_fault_revive() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0)"A")(data(i32.const 16)"B")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $a i32)(local $b i32)(local.set $a(call $i(i32.const 0)(i32.const 1)(i32.const 64)(i32.const 8)))(local.set $b(call $i(i32.const 16)(i32.const 1)(i32.const 80)(i32.const 8)))(i32.store8(i32.const 32)(local.get $a))(i32.store8(i32.const 33)(local.get $b))(drop(call $w(local.get $out)(i32.const 32)(i32.const 2)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 1024) "{h}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(if(i32.eq(i32.load8_u(local.get $rp))(i32.const 65))(then(unreachable)))(i32.store8(local.get $dp)(i32.const 50))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 1024)(i32.const {len})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const 0)))(if(i32.ne(local.get $n)(i32.const 2))(then(return(local.get $n))))(if(i32.ne(i32.load8_u(i32.const 3000))(i32.const 247))(then(return(i32.const -6))))(if(i32.ne(i32.load8_u(i32.const 3001))(i32.const 1))(then(return(i32.const -7))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-cb-fault", &root).unwrap(), Vec::<u8>::new());
}
