use super::basic::{hex, run};

#[test]
fn buffered_eval_repeated_calls_work() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "X")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#).unwrap();
    let h = hex(&p2);
    let root = format!(
        r#"(module(import "skrynia" "eval_buffered"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{h}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $i i32)(local $n i32)(block $done(loop $lp(br_if $done(i32.ge_s(local.get $i)(i32.const 5)))(local.set $n(call $e(i32.const 0)(i32.const {len})(i32.const 2048)(i32.const 0)(i32.const 3000)(i32.const 4)(i32.const -1)))(if(i32.ne(local.get $n)(i32.const 1))(then(return(local.get $n))))(if(i32.ne(i32.load8_u(i32.const 3000))(i32.const 88))(then(return(i32.const -6))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(i32.const 0)))"#,
        h = h,
        len = p2.len()
    );
    assert_eq!(run("buf-repeat", &root).unwrap(), Vec::<u8>::new());
}
