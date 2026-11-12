use crate::{Result, State, missing};
use axum::http::Method;

pub fn run(
    state: &State,
    path: &str,
    method: Method,
    token: &str,
    spec: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    match (parts.as_slice(), method) {
        (["grants"], Method::POST) => state.child(token, &spec.unwrap()),
        (["grants", id], Method::GET) => state.inspect(id, token),
        (["grants", id, "users"], Method::POST) => state.issue_user(id, token),
        (["grants", id, "users", user], Method::DELETE) => {
            state.revoke_user(id, token, user)?;
            Ok(serde_json::json!({"ok":true,"grantId":id,"userId":user}))
        }
        (["grants", id], Method::DELETE) => {
            state.revoke_grant(id, token)?;
            Ok(serde_json::json!({"ok":true,"grantId":id}))
        }
        _ => Err(missing("not_found")),
    }
}
