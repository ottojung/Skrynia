#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Notification {
    pub subscription_id: String,
    pub generation: u64,
    pub attempts: u32,
    pub next_at: u64,
}
