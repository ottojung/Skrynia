use super::QuotaIndex;
impl QuotaIndex {
    pub fn attach_grant(&mut self, id: &str, scope: &str) {
        let shared = self.scopes.shared(scope).expect("initialized grant scope");
        self.by_grant.insert(id.into(), shared);
    }
}
