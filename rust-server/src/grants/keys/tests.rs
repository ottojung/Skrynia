use super::scope::{matches, subset, valid_key, valid_scope};
#[test]
fn key_grammar() {
    for key in [
        "a",
        "a..b",
        "v3.loop.",
        "A_012",
        "site/releases/v1/assets/index-a1.js",
        "foo-bar",
    ] {
        assert!(valid_key(key));
    }
    for key in [
        "", ".a", "/a", "a/", "a//b", "a/../b", "a/.x", "á", "*", "a b",
    ] {
        assert!(!valid_key(key));
    }
    assert!(valid_key(&"x".repeat(257)));
    assert!(!valid_scope(&"x".repeat(258)));
}
#[test]
fn scope_attenuation() {
    for scope in [
        "",
        "*",
        "a.*",
        "alice.config",
        "a..*",
        "site/*",
        "site/assets/*",
    ] {
        assert!(valid_scope(scope));
    }
    for scope in [".hidden*", "a//b*", "a?", "**", "x/../*"] {
        assert!(!valid_scope(scope));
    }
    assert!(subset("a.b", "a.*"));
    assert!(subset("site/releases/*", "site/*"));
    assert!(matches("site/*", "site/v1/index.html"));
    assert!(!subset("a.*", "a.b"));
    assert!(subset("", "a.b"));
    assert!(matches("a.*", "a.config"));
    assert!(!matches("a.*", "b.config"));
}
