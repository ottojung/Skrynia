use super::key;
#[test]
fn key_path_does_not_use_form_semantics() {
    assert_eq!(key("valid.name", 256).unwrap(), "valid.name");
    assert_eq!(key("valid%2Ename", 256).unwrap(), "valid.name");
    for raw in [
        "valid&x=another",
        "v+plus",
        "%ZZ",
        "%C0%AF",
        "../evil",
        "%2F",
    ] {
        assert!(key(raw, 256).is_err(), "{raw}");
    }
    assert_eq!(
        key("bad%ZZ", 256).unwrap_err().1,
        "invalid_percent_encoding"
    );
}

#[test]
fn configured_key_limit_is_independent_of_valid_key_grammar() {
    assert!(key("abcdef", 5).is_err());
    assert_eq!(key("abcdef", 6).unwrap(), "abcdef");
    assert_eq!(key(&"x".repeat(257), 512).unwrap(), "x".repeat(257));
}
