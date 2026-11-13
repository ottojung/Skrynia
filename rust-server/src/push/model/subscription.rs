#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Subscription {
    pub id: String,
    pub grant_id: String,
    pub locations: String,
    pub events: Vec<String>,
    pub endpoint: String,
    pub keys: PushKeys,
    pub cap_hash: String,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}
