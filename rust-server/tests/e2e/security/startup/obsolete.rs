use std::process::Command;
#[test]
fn retired_push_allowlist_configuration_is_not_silently_ignored() {
    let root = std::env::temp_dir().join(format!("skrynia-config-{}", rand::random::<u64>()));
    let result = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
        .env("SKRYNIA_DATA_DIR", &root)
        .env("SKRYNIA_URL", "http://127.0.0.1:17380")
        .env("SKRYNIA_PUSH_ALLOWED_HOSTS", "push.example.org")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("obsolete SKRYNIA_PUSH_ALLOWED_HOSTS")
    );
    assert!(!root.exists(), "reject before creating persistent data");
}
