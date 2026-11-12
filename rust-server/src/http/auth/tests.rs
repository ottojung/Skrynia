use super::key;
#[test]
fn key_path_does_not_use_form_semantics() {
    assert_eq!(key("valid.name").unwrap(), "valid.name");
    assert_eq!(key("valid%2Ename").unwrap(), "valid.name");
    for raw in [
        "valid&x=another",
        "v+plus",
        "%ZZ",
        "%C0%AF",
        "../evil",
        "%2F",
    ] {
        assert!(key(raw).is_err(), "{raw}");
    }
    assert_eq!(key("bad%ZZ").unwrap_err().1, "invalid_percent_encoding");
}
