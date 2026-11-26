use super::call::call;

const NOOP: &str = "(func $cb (global.set $g(i64.const 1)))";

#[test]
fn two_imports_are_rejected() {
    let child = r#"(module(import "a" "x"(func))(import "b" "y"(func))(func(export "_start")))"#;
    assert_eq!(call("two", child, 0, 0, 0, NOOP).0, -1);
}

#[test]
fn non_function_import_is_rejected() {
    let child = r#"(module(import "a" "m"(memory 1))(func(export "_start")))"#;
    assert_eq!(call("memory", child, -1, 0, 0, NOOP).0, -1);
}

#[test]
fn non_integer_signature_is_rejected() {
    let child = r#"(module(import "a" "f"(func(param f32)))(func(export "_start")))"#;
    assert_eq!(call("float", child, 0, 0, 0, NOOP).0, -1);
}

#[test]
fn mismatched_signature_is_rejected() {
    let child = r#"(module(import "a" "f"(func(param i32)(result i32)))(func(export "_start")))"#;
    let supplied = "(func $cb(param i64)(result i64)(local.get 0))";
    assert_eq!(call("mismatch", child, 0, 0, 0, supplied).0, -1);
}

#[test]
fn supplied_function_without_import_is_rejected() {
    let child = "(module(func(export \"_start\")))";
    assert_eq!(call("unused", child, 0, 0, 0, NOOP).0, -1);
}
