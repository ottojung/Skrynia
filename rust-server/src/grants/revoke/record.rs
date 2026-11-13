#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revocation {
    pub id: String,
    pub ids: Vec<String>,
}
impl Revocation {
    pub fn valid(&self, name: &str) -> bool {
        let mut uniq = std::collections::HashSet::new();
        crate::valid_identifier(&self.id)
            && name == format!("{}.json", self.id)
            && self.ids.contains(&self.id)
            && self
                .ids
                .iter()
                .all(|id| crate::valid_identifier(id) && uniq.insert(id))
    }
}
