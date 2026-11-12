use super::generated;
#[test]
fn node_atomic_json_names_are_scoped_to_known_records() {
    let id = "a".repeat(32);
    let nonce = "b".repeat(24);
    let grant = format!("{id}.json.{nonce}.tmp");
    assert!(generated("state/grants", &grant));
    assert!(generated("state/push-subs", &grant));
    assert!(generated("state/push/outbox", &grant));
    assert!(generated("state/push", &format!("vapid.json.{nonce}.tmp")));
    assert!(!generated(
        "state/grants",
        &format!("vapid.json.{nonce}.tmp")
    ));
    assert!(!generated("state/push", &grant));
    assert!(!generated(
        "state/grants",
        &format!("{id}.json.{}.tmp", "z".repeat(24))
    ));
    assert!(!generated(
        "state/grants",
        &format!("{id}.json.{}.tmp", "b".repeat(25))
    ));
    assert!(!generated("state/grants", "record.json.abcd.tmp"));
}
