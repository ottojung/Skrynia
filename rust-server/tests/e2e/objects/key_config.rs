#[test]
fn invalid_key_length_configuration_prevents_startup() {
    for value in ["0", "256", "999999"] {
        let result = std::process::Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_URL", "http://127.0.0.1:17380")
            .env("SKRYNIA_MAX_KEY_LENGTH", value)
            .output()
            .unwrap();
        assert!(
            !result.status.success(),
            "accepted invalid max key length {value}"
        );
    }
}
