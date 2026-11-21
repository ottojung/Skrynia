use skrynia_server::safe;
use std::path::Path;

#[test]
fn rejects_bad_paths() {
    for p in ["/abs", "../x", "a/../../b", "a\0b", "", "a//b", "a/"] {
        assert!(safe::clean_relative(Path::new(p)).is_err(), "{p}");
    }
    use std::os::unix::ffi::OsStrExt;
    let bad = std::ffi::OsStr::from_bytes(b"a/\xff");
    assert!(safe::clean_relative(Path::new(bad)).is_err());
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
fn guest_trap_propagates_error() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1))))"#;
    let child = r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32) unreachable (i32.const 0)))"#;
    let dir = std::env::temp_dir().join("skrynia-v3-trap-prop");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert!(k.invoke(&wat::parse_str(child).unwrap()).is_err());
}
