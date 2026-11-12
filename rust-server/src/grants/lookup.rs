use super::Grant;
use super::keys::{matches, sha};
use crate::Snapshot;
use crate::{Result, forbidden, missing};

impl Snapshot {
    pub fn user(&self, token: &str) -> Result<&Grant> {
        let hash = sha(token.as_bytes());
        self.grants
            .values()
            .find(|g| g.users.iter().any(|u| u.hash == hash))
            .ok_or_else(|| forbidden("invalid_grant_user_token"))
    }
    pub fn issuer(&self, id: &str, token: &str) -> Result<&Grant> {
        let grant = self
            .grants
            .get(id)
            .ok_or_else(|| missing("grant_not_found"))?;
        if grant.issuer_hash != sha(token.as_bytes()) {
            return Err(forbidden("invalid_grant_issuer_token"));
        }
        Ok(grant)
    }
    pub fn grant_for(&self, token: &str, operation: &str, key: &str) -> Result<&Grant> {
        let grant = self.user(token)?;
        if !grant.operations.iter().any(|op| op == operation) {
            return Err(forbidden("grant_operation_forbidden"));
        }
        if !matches(&grant.locations, key) {
            return Err(forbidden("location_not_authorized"));
        }
        Ok(grant)
    }
    pub fn children_of(&self, ancestor: &str) -> Vec<&Grant> {
        self.grants
            .values()
            .filter(|candidate| {
                let mut next = Some(candidate.id.as_str());
                while let Some(id) = next {
                    if id == ancestor {
                        return true;
                    }
                    next = self.grants.get(id).and_then(|g| g.parent.as_deref());
                }
                false
            })
            .collect()
    }
}
