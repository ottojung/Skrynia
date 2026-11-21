use skrynia_server::boot;
use std::fs;

fn run(name: &str, root: &str) -> Vec<u8> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    let out = k.invoke(&[]).unwrap();
    let _ = fs::remove_dir_all(&dir);
    out
}

#[test]
fn release_requires_ownership_and_try_not_recursive() {
    let root = r#"(module(import "skrynia" "try_mutation_lock"(func $t(result i32)))
(import "skrynia" "release_mutation_lock"(func $r(result i32)))
(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param i32 i32)(result i32)
  (i32.store8(i32.const 0)(i32.add(i32.const 48)(i32.sub(i32.const 0)(call $r))))
  (i32.store8(i32.const 1)(i32.add(i32.const 48)(call $t)))
  (i32.store8(i32.const 2)(i32.add(i32.const 48)(call $t)))
  (i32.store8(i32.const 3)(i32.add(i32.const 48)(i32.sub(i32.const 0)(call $r))))
  (i32.store8(i32.const 4)(i32.add(i32.const 48)(i32.sub(i32.const 0)(call $r))))
  (drop(call $w(local.get 1)(i32.const 0)(i32.const 5)))(i32.const 0)))"#;
    let out = run("vm-mut-perm", root);
    assert_eq!(&out, b"41004");
}
