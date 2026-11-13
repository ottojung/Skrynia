use super::data_dir;
use std::{fs, process::Command};
#[test]
fn invalid_key_length_configuration_prevents_startup() {
    let home = data_dir();
    fs::create_dir(&home).unwrap();
    for value in ["0", "256", "999999"] {
        let result = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &home)
            .env("SKRYNIA_URL", "http://127.0.0.1:17380")
            .env("SKRYNIA_MAX_KEY_LENGTH", value)
            .output()
            .unwrap();
        assert!(!result.status.success(), "accepted invalid {value}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("invalid key length"), "{stderr}");
    }
    fs::remove_dir_all(home).unwrap();
}
