use crate::{Result, State, missing};
impl State {
    pub fn revoke_user(&self, id: &str, issuer: &str, user: &str) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        let mut grant = data.issuer(id, issuer)?.clone();
        let removed = grant
            .users
            .iter()
            .find(|u| u.id == user)
            .map(|u| u.hash.clone());
        let count = grant.users.len();
        grant.users.retain(|u| u.id != user);
        if count == grant.users.len() {
            return Err(missing("user_not_found"));
        }
        self.persist_json(
            &self.dir.join("state/grants").join(format!("{id}.json")),
            &grant,
        )?;
        data.grants.insert(id.into(), grant);
        if let Some(hash) = removed {
            data.tokens.remove(&hash);
        }
        Ok(())
    }
}
