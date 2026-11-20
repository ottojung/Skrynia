use skrynia_server::boot;
use std::fs;

fn k(name: &str, p1: &str) -> boot::Kernel {
    let d = std::env::temp_dir().join(format!("skrynia-p1dir-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(d.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    boot::Kernel::start_with(&d.join("r.wasm"), &d.join("data"), Some(&d.join("p1.wasm"))).unwrap()
}

#[test]
fn wrong_fd_direction_is_denied() {
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(if(i32.ne(call $w(local.get $i)(i32.const 0)(i32.const 1))(i32.const -4))(then(return(i32.const -1))))(if(i32.ne(call $r(local.get $o)(i32.const 0)(i32.const 1))(i32.const -4))(then(return(i32.const -2))))(i32.const 0)))"#;
    assert!(k("dir", p1).invoke(b"x").is_ok());
}
