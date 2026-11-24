use libskrynia::boot;
use std::fs;

fn kernel(name: &str, root: &str) -> (boot::Kernel, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    (
        boot::Kernel::start(
            &dir.join("r.wasm"),
            &dir.join("operator"),
            &dir.join("user"),
        )
        .unwrap(),
        dir,
    )
}

#[test]
fn trap_releases_lock_for_next_request() {
    let spin = r#"(module(import "skrynia" "try_mutation_lock"(func $t(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param i32 i32)(result i32)
  (if(i32.ne(call $t)(i32.const 1))(then(return(i32.const -10))))
  (loop $s(br $s))(i32.const 0)))"#;
    let ok = r#"(module(import "skrynia" "try_mutation_lock"(func $t(result i32)))
(import "skrynia" "release_mutation_lock"(func $r(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param i32 i32)(result i32)
  (if(i32.ne(call $t)(i32.const 1))(then(return(i32.const -11))))
  (if(i32.ne(call $r)(i32.const 0))(then(return(i32.const -12))))
  (i32.const 0)))"#;
    let (k, d) = kernel("vm-mut-trap", spin);
    assert!(k.invoke(&[]).is_err());
    drop(k);
    let (k, _) = kernel("vm-mut-trap", ok);
    k.invoke(&[]).unwrap();
    let _ = fs::remove_dir_all(d);
}
