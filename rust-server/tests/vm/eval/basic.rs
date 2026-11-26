use super::call::call;

#[test]
fn mixed_width_call_runs_in_originating_context() {
    let child = r#"(module
(import "app" "mixed" (func $f(param i32 i64)(result i64)))
(func (export "_start") (drop (call $f (i32.const 7)(i64.const 35)))))"#;
    let supplied = r#"(func $cb(param i32 i64)(result i64)
(global.set $g(i64.add(global.get $g)(local.get 1)))(local.get 1))"#;
    let (status, state) = call("mixed", child, 0, 0, 0, supplied);
    assert_eq!(status, 0);
    assert_eq!(
        state, 35,
        "supplied function must use its originating context"
    );
}

#[test]
fn zero_argument_zero_result_call() {
    let child = r#"(module
(import "app" "mark" (func $m))
(func (export "_start") (call $m)))"#;
    let supplied = "(func $cb (global.set $g(i64.const 1)))";
    assert_eq!(call("zero", child, 0, 0, 0, supplied), (0, 1));
}

#[test]
fn start_section_is_a_valid_entrypoint() {
    let child = r#"(module
(import "app" "mark" (func $m))
(func $init (call $m))
(start $init))"#;
    let supplied = "(func $cb (global.set $g(i64.const 1)))";
    assert_eq!(call("start", child, 0, 0, 0, supplied), (0, 1));
}
