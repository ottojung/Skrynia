use super::basic::{hex, run};

#[test]
fn buffered_child_gets_only_two_fds() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(if(i32.ne(call $r(i32.const 3)(i32.const 0)(i32.const 1))(i32.const -7))(then(return(i32.const -5))))(if(i32.ne(call $w(i32.const 99)(i32.const 0)(i32.const 1))(i32.const -7))(then(return(i32.const -6))))(i32.store8(i32.const 0)(i32.const 7))(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 3)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 512)(i32.const 0)(i32.const 768)(i32.const 8)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 1))(then(return(local.get $n))))(if(i32.ne(i32.load8_u(i32.const 768))(i32.const 7))(then(return(i32.const -8))))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-two-fds", &root).unwrap(), Vec::<u8>::new());
}
