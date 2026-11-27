use libskrynia::boot::Kernel;

fn with_root(wat: &str, f: impl FnOnce(Result<Kernel, String>)) {
    let d = std::env::temp_dir().join(format!(
        "skrynia-abi31-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    std::fs::create_dir_all(d.join("operator")).unwrap();
    std::fs::create_dir_all(d.join("user")).unwrap();
    let root = d.join("root.wasm");
    std::fs::write(&root, wat::parse_str(wat).unwrap()).unwrap();
    f(Kernel::start(&root, &d.join("operator"), &d.join("user")).map_err(|e| e.to_string()));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn bulk_bounds_eof_partial_progress_and_permission() {
    with_root(include_str!("../../test-fixtures/abi31/bulk.wat"), |k| {
        assert_eq!(k.unwrap().invoke(b"XYZ").unwrap(), b"XYZdefXYZd");
    });
}

#[test]
fn memoryless_scalar_root_and_legacy_import_rejection() {
    let root = r#"(module
        (import "skrynia_v3_1" "read1" (func $r (param i32) (result i32)))
        (import "skrynia_v3_1" "write1" (func $w (param i32 i32) (result i32)))
        (func (export "skrynia_main") (param i32 i32) (result i32)
            (call $w (local.get 1) (call $r (local.get 0)))))"#;
    with_root(root, |k| assert_eq!(k.unwrap().invoke(b"x").unwrap(), b"x"));
    with_root(&root.replace("skrynia_v3_1", "skrynia"), |k| {
        assert!(k.is_err())
    });
}

#[test]
fn missing_memory_is_error_before_transfer() {
    let root = r#"(module
        (import "skrynia_v3_1" "read" (func $read (param i32 i64 i64) (result i64)))
        (import "skrynia_v3_1" "read1" (func $r (param i32) (result i32)))
        (import "skrynia_v3_1" "write1" (func $w (param i32 i32) (result i32)))
        (func (export "skrynia_main") (param i32 i32) (result i32)
            (if (i64.ne (call $read (local.get 0) (i64.const 0) (i64.const 1)) (i64.const -3)) (then unreachable))
            (call $w (local.get 1) (call $r (local.get 0)))))"#;
    with_root(root, |k| assert_eq!(k.unwrap().invoke(b"x").unwrap(), b"x"));
}
