use super::fixture::Fixture;
use axum::http::StatusCode;
use std::sync::atomic::Ordering;
#[test]
fn previously_admitted_mutations_cannot_cross_failed_state() {
    let f = Fixture::new(1);
    f.state.failed.store(true, Ordering::Release);
    let unavailable = StatusCode::SERVICE_UNAVAILABLE;
    assert_eq!(f.state.require_healthy().unwrap_err().0, unavailable);
    assert_eq!(
        f.state
            .change("x", "create", b"bytes", None, "bad")
            .unwrap_err()
            .0,
        unavailable
    );
    assert_eq!(
        f.state.child("bad", &serde_json::json!({})).unwrap_err().0,
        unavailable
    );
    assert_eq!(f.state.issue_user("bad", "bad").unwrap_err().0, unavailable);
    assert_eq!(
        f.state.revoke_user("bad", "bad", "bad").unwrap_err().0,
        unavailable
    );
    assert_eq!(
        f.state.revoke_grant("bad", "bad").unwrap_err().0,
        unavailable
    );
    assert_eq!(f.state.inspect("bad", "bad").unwrap_err().0, unavailable);
    assert_eq!(
        f.state
            .update_subscription(
                &f.sub.id,
                "bad",
                &serde_json::json!({"endpoint":"https://push.example.org/new"})
            )
            .unwrap_err()
            .0,
        unavailable
    );
    assert_eq!(
        f.state.remove_subscription(&f.sub.id, "bad").unwrap_err().0,
        unavailable
    );
}
