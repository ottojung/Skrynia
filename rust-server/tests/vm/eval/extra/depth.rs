use skrynia_server::boot::Kernel;
use std::fs;

fn chain(wraps: usize) -> String {
    let leaf = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "D")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#;
    let mut wat = leaf.to_string();
    for _ in 0..wraps {
        let wasm = wat::parse_str(&wat).unwrap();
        let hex: String = wasm.iter().map(|b| format!("\\{b:02x}")).collect();
        wat = format!(
            r#"(module(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "{hex}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
            hex = hex,
            len = wasm.len()
        );
    }
    wat
}

fn run(name: &str, wraps: usize) -> Result<Vec<u8>, i32> {
    let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(chain(wraps)).unwrap()).unwrap();
    let k = Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&dir);
    r
}

#[test]
fn nested_depth_allows_15_levels() {
    assert_eq!(run("v3-depth-15", 15).unwrap(), b"D");
}

#[test]
fn nested_depth_rejects_16_levels() {
    // StoreLimits instances/memories cap (16) binds before the depth-32 check.
    assert_eq!(run("v3-depth-16", 16).unwrap_err(), -1);
}
