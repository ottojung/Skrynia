use super::fixture::Fixture;
#[test]
fn incomplete_unapplied_intent_never_notifies() {
    let f = Fixture::new();
    f.record(&"a".repeat(32), "new_key", "create", None, false);
    assert_eq!(f.replay().unwrap(), 0);
    assert_eq!(f.transactions(), 0);
    assert!(!f.root.join("store/new_key").exists());
}
#[test]
fn applied_mutation_notifies_once_after_restart() {
    let f = Fixture::new();
    f.record(&"b".repeat(32), "new_key", "create", Some(b"new"), false);
    assert_eq!(f.replay().unwrap(), 1);
    assert_eq!(f.replay().unwrap(), 0);
    assert_eq!(f.transactions(), 0);
}
#[test]
fn committed_intent_replays_even_if_file_later_changes() {
    let f = Fixture::new();
    f.record(
        &"c".repeat(32),
        "old_key",
        "replace",
        Some(b"changed"),
        true,
    );
    assert_eq!(f.replay().unwrap(), 1);
    assert_eq!(f.transactions(), 0);
}
#[test]
fn invalid_intent_preserves_record_and_fails_closed() {
    let f = Fixture::new();
    f.record(&"d".repeat(32), "a_key", "invalid", None, false);
    assert!(f.replay().is_err());
    assert_eq!(f.transactions(), 1);
}
#[test]
fn delete_intent_replays_when_object_is_absent() {
    let f = Fixture::new();
    f.record(&"e".repeat(32), "gone_key", "delete", None, false);
    assert_eq!(f.replay().unwrap(), 1);
    assert_eq!(f.transactions(), 0);
}
