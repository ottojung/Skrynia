use crate::{Result, State, forbidden};
impl State {
    pub fn inspect(&self, id: &str, token: &str) -> Result<serde_json::Value> {
        let data = self.data.lock().map_err(|_| crate::internal())?;
        let grant = data.user(token)?;
        if grant.id != id {
            return Err(forbidden("invalid_grant_user_token"));
        }
        let mut value = serde_json::to_value(grant).map_err(|_| crate::internal())?;
        let map = value.as_object_mut().unwrap();
        map.remove("issuerHash");
        map.remove("rootIssuerToken");
        map.remove("users");
        map.insert("userCount".into(), serde_json::json!(grant.users.len()));
        let (bytes, objects) = data.usage(id, None);
        map.insert(
            "usage".into(),
            serde_json::json!({"bytes":bytes,"objects":objects}),
        );
        Ok(value)
    }
}
