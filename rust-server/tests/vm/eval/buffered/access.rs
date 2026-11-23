use super::basic::{hex, run};

#[test]
fn untrusted_leaf_cannot_import_eval_buffered() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-no-leaf", &root).unwrap_err(), -1);
}

#[test]
fn eval_buffered_requires_eval_delegated_context() {
    let p1 = wat::parse_str(r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(i32.const 0)))"#).unwrap();
    let h = hex(&p1);
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        h = h,
        len = p1.len()
    );
    assert_eq!(run("buf-no-delegate", &root).unwrap_err(), -1);
}

#[test]
fn root_can_use_eval_buffered() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "R")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 0)(i32.const 768)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 1))(then(return(local.get $n))))(if(i32.ne(i32.load8_u(i32.const 768))(i32.const 82))(then(return(i32.const -9))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-root-ok", &root).unwrap(), Vec::<u8>::new());
}
