use skrynia_server::safe;
use std::path::Path;

#[test]
fn rejects_bad_paths() {
    assert!(safe::clean_relative(Path::new("/abs")).is_err());
    assert!(safe::clean_relative(Path::new("../x")).is_err());
    assert!(safe::clean_relative(Path::new("a/../../b")).is_err());
    assert!(safe::clean_relative(Path::new("a\0b")).is_err());
    assert!(safe::clean_relative(Path::new("")).is_err());
    assert!(safe::clean_relative(Path::new("a//b")).is_err());
    assert!(safe::clean_relative(Path::new("a/")).is_err());
    use std::os::unix::ffi::OsStrExt;
    assert!(safe::clean_relative(Path::new(std::ffi::OsStr::from_bytes(b"a/\xff"))).is_err());
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
fn nested_eval_overflow_visible() {
    let c2 = wat::parse_str(r#"(module(memory 1)(export "memory"(memory 0))(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32) i32.const 3))"#).unwrap();
    let data: String = c2.iter().map(|b| format!("\\{:02x}", b)).collect();
    let c1 = format!(
        r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "{data}")(func(export "skrynia_main")(param $ip i32)(param $il i32)(param $op i32)(param $oc i32)(result i32)(call $i(i32.const 0)(i32.const {len})(local.get $op)(local.get $oc))))"#,
        data = data,
        len = c2.len()
    );
    let root = include_str!("../test-fixtures/root_safe_nested.wat");
    let dir = std::env::temp_dir().join("skrynia-v3-nest-ov");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = skrynia_server::boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    assert_eq!(k.invoke(&wat::parse_str(&c1).unwrap()).unwrap(), b"1");
}
