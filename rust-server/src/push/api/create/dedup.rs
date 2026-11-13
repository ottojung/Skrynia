use crate::{Fail, PushInput, Result, State, Subscription, forbidden};
use axum::http::StatusCode;
pub fn same_events(a: &[String], b: &[String]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    a == b
}
pub fn rotate(
    state: &State,
    sub: &mut Subscription,
    input: &PushInput,
    cap: Option<&str>,
) -> Result<String> {
    let provided = cap.ok_or(Fail(StatusCode::FORBIDDEN, "capability_required"))?;
    if !crate::secure_match(&sub.cap_hash, &crate::sha(provided.as_bytes())) {
        return Err(forbidden("invalid_capability"));
    }
    let capability = crate::random_secret();
    sub.cap_hash = crate::sha(capability.as_bytes());
    sub.keys = input.keys.clone();
    super::persist(state, sub)?;
    Ok(capability)
}
