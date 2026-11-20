use skrynia_server::boot;
use std::fs;
#[test]
fn program1_may_not_import_seek_truncate_or_sync() {
    let path = std::env::temp_dir().join(format!(
        "skrynia-program1-imports-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    fs::create_dir_all(path.join("data")).unwrap();
    let root = r#"(module (memory(export "memory") 1)
      (func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    fs::write(path.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    for (name, sig) in [
        ("seek", "(param i32 i64)"),
        ("truncate", "(param i32 i64)"),
        ("sync", "(param i32)"),
    ] {
        let guest = format!(
            r#"(module
          (import "skrynia" "{name}"(func {sig}(result i32)))
          (memory(export "memory") 1)
          (func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#
        );
        fs::write(path.join("p1.wasm"), wat::parse_str(&guest).unwrap()).unwrap();
        let err = boot::Kernel::start_with(
            &path.join("root.wasm"),
            &path.join("data"),
            Some(&path.join("p1.wasm")),
        );
        assert!(err.is_err(), "{name} must not be importable by Program 1");
    }
    fs::remove_dir_all(path).unwrap();
}
