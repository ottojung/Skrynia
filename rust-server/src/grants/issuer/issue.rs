use super::User;
use super::secret;
use crate::random_id;
use crate::sha;
use crate::{Result, State};
impl State {
    pub fn issue_user(&self, id: &str, issuer: &str) -> Result<serde_json::Value> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        self.require_healthy()?;
        let mut grant = data.issuer(id, issuer)?.clone();
        if !grant.users.is_empty() {
            data.check_metadata(id, 128)?;
        }
        let token = secret();
        let user = User {
            id: random_id(),
            hash: sha(token.as_bytes()),
        };
        if data.tokens.contains_key(&user.hash) {
            return Err(crate::internal());
        }
        grant.users.push(user.clone());
        self.persist_json(
            &self.dir.join("state/grants").join(format!("{id}.json")),
            &grant,
        )?;
        data.grants.insert(id.into(), grant);
        data.tokens.insert(user.hash, id.into());
        Ok(serde_json::json!({"grantId":id,"userId":user.id,"userToken":token}))
    }
}
