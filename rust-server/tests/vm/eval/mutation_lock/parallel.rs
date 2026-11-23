use skrynia_server::boot;
use std::{fs, sync::Arc, thread};
fn kernel(name: &str, root: &str) -> (boot::Kernel, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
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
fn parallel_try_enforces_exclusion() {
    let root = r#"(module(import "skrynia" "try_mutation_lock"(func $t(result i32)))(import "skrynia" "release_mutation_lock"(func $r(result i32)))(import "skrynia" "open_ram"(func $or(result i32)))(import "skrynia" "cas_ram_u64"(func $c(param i32 i64 i64 i64 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(local $fd i32)(if(i32.eqz(call $t))(then(i32.store8(i32.const 0)(i32.const 48))(drop(call $w(local.get 1)(i32.const 0)(i32.const 1)))(return(i32.const 0))))(local.set $fd(call $or))(drop(call $c(local.get $fd)(i64.const 0)(i64.const 0)(i64.const 1)(i32.const 64)))(if(i64.ne(i64.load(i32.const 64))(i64.const 0))(then(return(i32.const -13))))(drop(call $c(local.get $fd)(i64.const 0)(i64.const 1)(i64.const 0)(i32.const 64)))(if(i64.ne(i64.load(i32.const 64))(i64.const 1))(then(return(i32.const -14))))(if(i32.ne(call $r)(i32.const 0))(then(return(i32.const -15))))(i32.store8(i32.const 0)(i32.const 49))(drop(call $w(local.get 1)(i32.const 0)(i32.const 1)))(i32.const 0)))"#;
    let (k, d) = kernel("vm-mut-par", root);
    k.ram().truncate(16).unwrap();
    let k = Arc::new(k);
    let mut hs = Vec::new();
    for _ in 0..8 {
        let k = k.clone();
        hs.push(thread::spawn(move || {
            for _ in 0..50 {
                let out = k.invoke(&[]).unwrap();
                assert!(out == b"0" || out == b"1");
            }
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    let _ = std::fs::remove_dir_all(d);
}
