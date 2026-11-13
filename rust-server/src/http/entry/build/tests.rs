use super::valid;
#[test]
fn release_provenance_requires_complete_version_and_40_to_64_hex_commit() {
    assert!(valid("development", "development"));
    assert!(valid("1.2.3", &"a".repeat(40)));
    assert!(valid("release", &"F".repeat(64)));
    assert!(!valid("", "development"));
    assert!(!valid("1.2.3", "development"));
    assert!(!valid("release", &"a".repeat(39)));
    assert!(!valid("release", &"a".repeat(65)));
    assert!(!valid("release", &"z".repeat(40)));
}
