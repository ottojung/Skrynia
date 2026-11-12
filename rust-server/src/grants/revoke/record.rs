#[derive(serde::Serialize, serde::Deserialize)]
pub struct Revocation {
    pub id: String,
    pub ids: Vec<String>,
}
impl Revocation {
    pub fn valid(&self, name: &str) -> bool {
        let valid_id = |s: &str| {
            s.len() == 32
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        };
        let mut uniq = std::collections::HashSet::new();
        valid_id(&self.id)
            && name == format!("{}.json", self.id)
            && self.ids.contains(&self.id)
            && self.ids.iter().all(|id| valid_id(id) && uniq.insert(id))
    }
}
