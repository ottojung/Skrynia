use skrynia_server::boot;
use std::fs;

fn run(name: &str, root: &str, body: &[u8]) -> Result<Vec<u8>, i32> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("r.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm, wat::parse_str(root).unwrap()).unwrap();
    let k = boot::Kernel::start(&wasm, &data).unwrap();
    let out = k.invoke(body);
    let _ = fs::remove_dir_all(&dir);
    out
}

#[test]
fn root_echoes_input() {
    let root = r#"(module (memory (export "memory") 1)
      (func (export "skrynia_main") (param $i i32) (param $l i32) (param $o i32) (param $c i32) (result i32)
        (memory.copy (local.get $o) (local.get $i) (local.get $l))
        (local.get $l)))"#;
    assert_eq!(run("v3-echo", root, b"hi").unwrap(), b"hi");
}

#[test]
fn eval_rejects_malicious_import() {
    let root = r#"(module
      (import "skrynia" "eval" (func $eval (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
      (memory (export "memory") 2)
      (func (export "skrynia_main") (param $in i32) (param $il i32) (param $out i32) (param $oc i32) (result i32)
        (local $r i32)
        (local.set $r (call $eval (local.get $in) (local.get $il) (i32.const 65536) (i32.const 0) (i32.const -1) (i32.const 70000) (i32.const 16)))
        (i32.store8 (local.get $out) (i32.add (i32.const 48) (i32.eq (local.get $r) (i32.const -1))))
        (i32.const 1)))"#;
    let child = r#"(module
      (import "wasi_snapshot_preview1" "fd_write" (func (param i32 i32 i32 i32) (result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32) (i32.const 0)))"#;
    assert_eq!(
        run("v3-mal", root, &wat::parse_str(child).unwrap()).unwrap(),
        b"1"
    );
}
