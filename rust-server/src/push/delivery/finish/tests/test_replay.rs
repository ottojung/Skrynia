use super::Fixture;
use std::fs;
use std::sync::atomic::Ordering;
#[test]
fn malformed_pending_journal_fails_closed_before_delivery_recovery() {
    let f = Fixture::new(1);
    let dir = f.path.join("state/store-transactions");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("invalid-name.json"), b"{}").unwrap();
    assert!(f.state.recover_journal().is_err());
    assert!(f.state.failed.load(Ordering::Acquire));
    let retry = f.state.recover_journal().unwrap_err();
    assert_eq!(retry.to_string(), "recovery required");
    assert_eq!(
        f.state
            .change("blocked", crate::Kind::Create, b"x", None, "bad")
            .unwrap_err()
            .0,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn failed_replay_of_applied_mutation_must_block_new_writes() {
    let f = Fixture::new(1);
    let dir = f.path.join("state/store-transactions");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.json", "b".repeat(32)));
    let intent = serde_json::json!({
        "key":"pending", "kind":"create", "sha256":crate::sha(b"x"), "committed":true
    });
    fs::write(&path, serde_json::to_vec(&intent).unwrap()).unwrap();
    let outbox = f.path.join("state/push/outbox.json");
    fs::remove_file(&outbox).unwrap();
    fs::create_dir(&outbox).unwrap();
    assert!(f.state.recover_journal().is_err());
    assert!(f.state.failed.load(Ordering::Acquire));
    assert!(path.exists(), "journal is still needed for recovery");
}
