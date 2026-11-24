use libskrynia::boot::Kernel;
use std::fs;
fn run(name: &str, root: &str) -> Result<Vec<u8>, i32> {
    let d = std::env::temp_dir().join(format!("sk-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&d);
    r
}
fn data(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("\\{b:02x}")).collect()
}
#[test]
fn child_cannot_use_parent_fd_leaked_by_callback() {
    let p2 = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $fd i32)(drop(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 4)))(local.set $fd(i32.load(i32.const 64)))(i32.store8(i32.const 96)(i32.eq(call $r(local.get $fd)(i32.const 128)(i32.const 1))(i32.const -7)))(drop(call $w(local.get $out)(i32.const 96)(i32.const 1)))(i32.const 0)))"#).unwrap();
    let p1 = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 1024) "{}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(i32.store(local.get $dp)(i32.load(i32.const 0)))(i32.const 4))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(i32.store(i32.const 0)(local.get $in))(call $e(i32.const 1024)(i32.const {})(local.get $in)(local.get $out)(i32.const 0))))"#,
        data(&p2),
        p2.len()
    );
    let p1b = wat::parse_str(&p1).unwrap();
    let root = format!(
        r#"(module(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {})(local.get $in)(local.get $out)(i32.const -1))))"#,
        data(&p1b),
        p1b.len()
    );
    assert_eq!(run("med-theft", &root).unwrap(), vec![1]);
}
