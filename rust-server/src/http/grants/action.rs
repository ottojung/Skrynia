use axum::http::Method;
pub enum Action {
    Create,
    Inspect(String),
    Issue(String),
    RevokeUser(String, String),
    Revoke(String),
}
impl Action {
    pub fn parse(path: &str, method: &Method) -> Option<Self> {
        let parts = path.split('/').collect::<Vec<_>>();
        match (parts.as_slice(), method) {
            (["", "grants"], &Method::POST) => Some(Self::Create),
            (["", "grants", id], &Method::GET) if !id.is_empty() => {
                Some(Self::Inspect((*id).into()))
            }
            (["", "grants", id, "users"], &Method::POST) if !id.is_empty() => {
                Some(Self::Issue((*id).into()))
            }
            (["", "grants", id, "users", user], &Method::DELETE)
                if !id.is_empty() && !user.is_empty() =>
            {
                Some(Self::RevokeUser((*id).into(), (*user).into()))
            }
            (["", "grants", id], &Method::DELETE) if !id.is_empty() => {
                Some(Self::Revoke((*id).into()))
            }
            _ => None,
        }
    }
    pub fn issuer_only(&self) -> bool {
        matches!(
            self,
            Self::Issue(_) | Self::RevokeUser(_, _) | Self::Revoke(_)
        )
    }
}
