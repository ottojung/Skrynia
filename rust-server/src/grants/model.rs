#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub id: String,
    pub parent: Option<String>,
    pub locations: String,
    pub operations: Vec<String>,
    pub max_bytes: Option<u64>,
    pub max_objects: Option<u64>,
    pub issuer_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_issuer_token: Option<String>,
    pub users: Vec<User>,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub id: String,
    pub hash: String,
}
