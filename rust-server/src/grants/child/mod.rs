mod scope;
mod spec;
use super::Grant;
use super::keys::{random_id, secret, sha};
use crate::{Result, State};

impl State {
    pub fn child(&self, token: &str, value: &serde_json::Value) -> Result<serde_json::Value> {
        let (mut data, spec, totals) = scope::checked(self, token, value)?;
        let parent = data.user(token)?.clone();
        let issuer_token = secret();
        let child = Grant {
            id: random_id(),
            parent: Some(parent.id),
            locations: spec.locations.clone(),
            operations: spec.operations.clone(),
            max_bytes: Some(spec.max_bytes),
            max_objects: Some(spec.max_objects),
            issuer_hash: sha(issuer_token.as_bytes()),
            root_issuer_token: None,
            users: vec![],
        };
        self.persist_json(
            &self
                .dir
                .join("state/grants")
                .join(format!("{}.json", child.id)),
            &child,
        )?;
        data.grants.insert(child.id.clone(), child.clone());
        scope::attach(&mut data, &child, totals);
        data.change_meta(
            &child.id,
            crate::Usage {
                bytes: 2048,
                objects: 0,
            },
            true,
        );
        Ok(
            serde_json::json!({"id":child.id,"issuerToken":issuer_token,"locations":spec.locations,
            "operations":spec.operations,"maxBytes":spec.max_bytes,"maxObjects":spec.max_objects}),
        )
    }
}
