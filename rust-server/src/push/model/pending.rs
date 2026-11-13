use super::Notification;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pending {
    pub generation: u64,
    pub attempts: u32,
    pub next_at: u64,
}
impl Pending {
    pub fn from_notice(notice: &Notification) -> Self {
        Self {
            generation: notice.generation,
            attempts: notice.attempts,
            next_at: notice.next_at,
        }
    }
    pub fn into_notice(self, id: String) -> Notification {
        Notification {
            subscription_id: id,
            generation: self.generation,
            attempts: self.attempts,
            next_at: self.next_at,
        }
    }
}
