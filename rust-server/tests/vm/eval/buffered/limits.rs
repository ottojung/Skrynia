use super::basic::{hex, run};

#[test]
fn buffered_output_overflow_returns_limit() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "12345")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $w(local.get $out)(i32.const 0)(i32.const 5)))(if(i32.eq(local.get $n)(i32.const -10))(then(return(i32.const -10))))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 0)(i32.const 768)(i32.const 4)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const -10))(then(return(i32.const -5))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-overflow", &root).unwrap(), Vec::<u8>::new());
}

#[test]
fn buffered_res_cap_overflow_returns_err() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "12345")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 5)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 0)(i32.const 768)(i32.const 67108865)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const -10))(then(return(i32.const -5))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-cap-limit", &root).unwrap(), Vec::<u8>::new());
}

#[test]
fn buffered_req_len_over_limit_rejected() {
    let p2 = wat::parse_str(r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 0)(i32.const 67108865)(i32.const 768)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const -10))(then(return(i32.const -5))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-req-limit", &root).unwrap(), Vec::<u8>::new());
}

#[test]
fn buffered_exact_cap_and_zero_req() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 16) "ABCD")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 8)))(if(i32.ne(local.get $n)(i32.const 0))(then(return(i32.const -5))))(local.set $n(call $w(local.get $out)(i32.const 16)(i32.const 4)))(if(i32.ne(local.get $n)(i32.const 4))(then(return(local.get $n))))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 0)(i32.const 768)(i32.const 4)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 4))(then(return(local.get $n))))(if(i32.ne(i32.load(i32.const 768))(i32.const 0x44434241))(then(return(i32.const -6))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-exact-cap", &root).unwrap(), Vec::<u8>::new());
}
