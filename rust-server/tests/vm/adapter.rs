pub fn adapt(root: &str) -> String {
    let transformed = root
        .replacen(
            "(module",
            r#"(module
            (import "skrynia" "read" (func $input_read (param i32 i32 i32) (result i32)))
            (import "skrynia" "write" (func $output_write (param i32 i32 i32) (result i32)))"#,
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
