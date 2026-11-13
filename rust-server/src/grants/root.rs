use super::Grant;
use super::keys::{random_id, secret, sha};
use crate::{State, write_json};
use std::io;

pub const OPS: [&str; 6] = [
    "create",
    "modify",
    "delete",
    "list",
    "delegate",
    "subscribe",
];
impl State {
    pub fn ensure_root(&self) -> io::Result<()> {
        let mut data = self.data.lock().map_err(|_| io::Error::other("poisoned"))?;
        if data.grants.values().any(|g| g.parent.is_none()) {
            return Ok(());
        }
        if !data.grants.is_empty() {
            return Err(io::Error::other("orphan grants"));
        }
        let token = secret();
        let grant = Grant {
            id: random_id(),
            parent: None,
            locations: "*".into(),
            operations: OPS.map(str::to_string).into(),
            max_bytes: None,
            max_objects: None,
            issuer_hash: sha(token.as_bytes()),
            root_issuer_token: Some(token),
            users: vec![],
        };
        let path = self
            .dir
            .join("state/grants")
            .join(format!("{}.json", grant.id));
        write_json(&path, &grant)?;
        data.grants.insert(grant.id.clone(), grant);
        Ok(())
    }
}
