#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pending {
    pub generation: u64,
    pub attempts: u32,
    pub next_at: u64,
}
