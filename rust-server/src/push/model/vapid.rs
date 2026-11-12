#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vapid {
    pub public_key: String,
    pub private_key: String,
}
