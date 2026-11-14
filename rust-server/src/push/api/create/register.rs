use crate::{Fail, PushInput, Result, Snapshot, State, Subscription, forbidden};
use axum::http::StatusCode;
pub fn create(
    state: &State,
    data: &mut Snapshot,
    input: PushInput,
    token: Option<&str>,
) -> Result<serde_json::Value> {
    if data.push.subscriptions.len() >= state.push_config.max_subscriptions {
        return Err(Fail(StatusCode::INSUFFICIENT_STORAGE, "subscriptions_full"));
    }
    let token = token
        .filter(|s| crate::valid_sha256(s))
        .ok_or(Fail(StatusCode::UNAUTHORIZED, "grant_user_token_required"))?;
    let grant = data.user(token)?;
    if !grant.operations.iter().any(|op| op == "subscribe") {
        return Err(forbidden("grant_operation_forbidden"));
    }
    if !crate::subset(&input.locations, &grant.locations) {
        return Err(forbidden("locations_outside_parent"));
    }
    let grant_id = grant.id.clone();
    data.check_capacity(&grant_id, 4096, 1)?;
    let capability = crate::random_secret();
    let sub = Subscription {
        id: crate::random_id(),
        grant_id,
        locations: input.locations,
        events: input.events,
        endpoint: input.endpoint,
        keys: input.keys,
        cap_hash: crate::sha(capability.as_bytes()),
    };
    super::persist(state, &sub)?;
    data.push.put_sub(sub.clone());
    data.change_meta(
        &sub.grant_id,
        crate::Usage {
            bytes: 4096,
            objects: 1,
        },
        true,
    );
    Ok(serde_json::json!({"id":sub.id,"capability":capability,"deduped":false}))
}
