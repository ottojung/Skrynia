mod inspect;
use super::keys::{random_id, secret, sha};
use super::model::User;
use crate::{Result, State, missing, write_json};

impl State {
    pub fn issue_user(&self, id: &str, issuer: &str) -> Result<serde_json::Value> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        let mut grant = data.issuer(id, issuer)?.clone();
        if !grant.users.is_empty() {
            data.check_metadata(id, 128)?;
        }
        let token = secret();
        let user = User {
            id: random_id(),
            hash: sha(token.as_bytes()),
        };
        grant.users.push(user.clone());
        write_json(
            &self.dir.join("state/grants").join(format!("{id}.json")),
            &grant,
        )
        .map_err(|_| crate::internal())?;
        data.grants.insert(id.into(), grant);
        Ok(serde_json::json!({"grantId":id,"userId":user.id,"userToken":token}))
    }
    pub fn revoke_user(&self, id: &str, issuer: &str, user: &str) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        let mut grant = data.issuer(id, issuer)?.clone();
        let count = grant.users.len();
        grant.users.retain(|u| u.id != user);
        if count == grant.users.len() {
            return Err(missing("user_not_found"));
        }
        write_json(
            &self.dir.join("state/grants").join(format!("{id}.json")),
            &grant,
        )
        .map_err(|_| crate::internal())?;
        data.grants.insert(id.into(), grant);
        Ok(())
    }
}
