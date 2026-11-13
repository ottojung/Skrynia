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
            .change("blocked", "create", b"x", None, "bad")
            .unwrap_err()
            .0,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}
