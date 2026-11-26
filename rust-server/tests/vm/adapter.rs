pub fn adapt(root: &str) -> String {
    let transformed = root
        .replacen(
            "(module",
            r#"(module
            (import "skrynia" "read1" (func $sk_read1_1 (param i32) (result i32)))
            (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $input_read (param $fd i32)(param $dst i32)(param $len i32)(result i32)(local $i i32)(local $b i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $b(call $sk_read1_1(local.get $fd)))(if(i32.lt_s(local.get $b)(i32.const 0))(then(if(i32.ne(local.get $b)(i32.const -1))(then(return(local.get $b))))(br $done)))(i32.store8(i32.add(local.get $dst)(local.get $i))(local.get $b))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))(func $output_write (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))"#,
            1,
        )
        .replace(r#"(func(export "skrynia_main")"#, "(func $old_entry");
    let end = transformed.rfind(')').unwrap();
    let wrapper = r#"(func (export "skrynia_main") (param $input i32)
        (param $output i32) (result i32)
        (local $n i32) (local $result i32)
        (local.set $n (call $input_read (local.get $input) (i32.const 32768) (i32.const 32768)))
        (if (i32.lt_s (local.get $n) (i32.const 0)) (then (return (local.get $n))))
        (local.set $result (call $old_entry (i32.const 32768)
            (local.get $n) (i32.const 60000) (i32.const 4096)))
        (if (i32.lt_s (local.get $result) (i32.const 0))
            (then (return (local.get $result))))
        (if (i32.ne (call $output_write (local.get $output)
                    (i32.const 60000) (local.get $result)) (local.get $result))
            (then (return (i32.const -5))))
        (i32.const 0))"#;
    format!("{}{}{}", &transformed[..end], wrapper, &transformed[end..])
}
