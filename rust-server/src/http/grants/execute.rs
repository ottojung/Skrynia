use super::action::Action;
use crate::{Result, State, missing};
pub fn run(
    state: &State,
    action: Action,
    token: &str,
    spec: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    match action {
        Action::Create => state.child(token, &spec.ok_or_else(|| missing("not_found"))?),
        Action::Inspect(id) => state.inspect(&id, token),
        Action::Issue(id) => state.issue_user(&id, token),
        Action::RevokeUser(id, user) => {
            state.revoke_user(&id, token, &user)?;
            Ok(serde_json::json!({"ok":true,"grantId":id,"userId":user}))
        }
        Action::Revoke(id) => {
            state.revoke_grant(&id, token)?;
            Ok(serde_json::json!({"ok":true,"grantId":id}))
        }
    }
}
