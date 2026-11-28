use super::support::start_at;

#[test]
fn private_path_preflight_and_cross_token_rename() {
    let dir = std::env::temp_dir().join(format!("abi31-rename-{}", rand::random::<u64>()));
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    std::fs::write(dir.join("operator/private"), b"private").unwrap();
    let guest = r#"(module
        (import "skrynia_v3_1" "rename" (func $rn (param i64 i64 i64 i64) (result i32)))
        (import "skrynia_v3_1" "exists" (func $ex (param i64 i64) (result i32)))
        (memory (export "memory") 1)
        (data (i32.const 0) "operator/private")
        (data (i32.const 32) "user/private")
        (func (export "skrynia_main") (param i32 i32) (result i32)
            (if (i32.ne (call $rn (i64.const 0) (i64.const 16) (i64.const 65535) (i64.const 2)) (i32.const -7)) (then unreachable))
            (if (i32.ne (call $rn (i64.const 0) (i64.const 16) (i64.const 32) (i64.const 12)) (i32.const -3)) (then unreachable))
            (if (i32.ne (call $ex (i64.const 0) (i64.const 4106)) (i32.const -3)) (then unreachable))
            (if (i32.ne (call $ex (i64.const 65000) (i64.const 4106)) (i32.const -7)) (then unreachable))
            (i32.const 0)))"#;
    let k = start_at(&dir, guest);
    k.invoke(&[]).unwrap();
    assert_eq!(
        std::fs::read(dir.join("operator/private")).unwrap(),
        b"private"
    );
    assert!(!dir.join("user/private").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}
