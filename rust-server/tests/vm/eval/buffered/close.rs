use super::basic::{hex, run};

#[test]
fn buffered_eval_captures_output_after_guest_close() {
    let p2 = wat::parse_str(
        r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "OK")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 2)))(drop(call $c(local.get $out)))(i32.const 0)))"#,
    )
    .unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 2))(then(return(local.get $n))))(if(i32.ne(i32.load16_u(i32.const 3000))(i32.const 0x4b4f))(then(return(i32.const -6))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-close-out", &root).unwrap(), Vec::<u8>::new());
}
