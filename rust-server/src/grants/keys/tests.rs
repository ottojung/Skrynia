use super::scope::{matches, subset, valid_key, valid_scope};
#[test]
fn key_grammar() {
    for key in ["a", "a..b", "v3.loop.", "A_012"] {
        assert!(valid_key(key));
    }
    for key in ["", ".a", "a/b", "a-b", "á", "*", "a b"] {
        assert!(!valid_key(key));
    }
    assert!(valid_key(&"x".repeat(257)));
    assert!(!valid_scope(&"x".repeat(258)));
}
#[test]
fn scope_attenuation() {
    for scope in ["", "*", "a.*", "alice.config", "a..*"] {
        assert!(valid_scope(scope));
    }
    for scope in [".hidden*", "a/b*", "a?", "**"] {
        assert!(!valid_scope(scope));
    }
    assert!(subset("a.b", "a.*"));
    assert!(!subset("a.*", "a.b"));
    assert!(subset("", "a.b"));
    assert!(matches("a.*", "a.config"));
    assert!(!matches("a.*", "b.config"));
}
