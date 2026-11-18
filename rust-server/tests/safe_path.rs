use skrynia_server::safe;
use std::path::Path;

#[test]
fn rejects_bad_paths() {
    assert!(safe::clean_relative(Path::new("/abs")).is_err());
    assert!(safe::clean_relative(Path::new("../x")).is_err());
    assert!(safe::clean_relative(Path::new("a/../../b")).is_err());
    assert!(safe::clean_relative(Path::new("a\0b")).is_err());
    assert!(safe::clean_relative(Path::new("")).is_err());
}

#[test]
fn accepts_normal_paths() {
    let p = safe::clean_relative(Path::new("a/b/c.txt")).unwrap();
    assert_eq!(p, Path::new("a/b/c.txt"));
    let j = safe::join(Path::new("/data"), Path::new("x/y")).unwrap();
    assert_eq!(j, Path::new("/data/x/y"));
}

#[test]
fn time_and_random_look_valid() {
    assert!(skrynia_server::sys::clock::now_millis() > 1_700_000_000_000);
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    skrynia_server::sys::rand::fill_secure(&mut a).unwrap();
    skrynia_server::sys::rand::fill_secure(&mut b).unwrap();
    assert_ne!(a, [0u8; 32]);
    assert_ne!(a, b);
}

#[test]
fn eval_overflow_is_reported() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_output_len"(func $l(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $il i32)(param $o i32)(param $oc i32)(result i32)(local $r i32)(local.set $r(call $e(local.get $i)(local.get $il)(i32.const 65536)(i32.const 0)(i32.const -1)(i32.const 70000)(i32.const 1)))(i32.store8(local.get $o)(i32.eq(local.get $r)(i32.const -11)))(i32.store8(i32.add(local.get $o)(i32.const 1))(i32.eq(call $l)(i32.const 5)))(i32.const 2)))"#;
    let child = r#"(module(memory(export "memory") 1)(data(i32.const 0) "hello")(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(i32.const 5)))"#;
    let dir = std::env::temp_dir().join("skrynia-v3-overflow");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(
        k.invoke(&wat::parse_str(child).unwrap()).unwrap(),
        vec![1, 1]
    );
}
