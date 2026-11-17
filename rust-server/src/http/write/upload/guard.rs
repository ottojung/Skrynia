use crate::Staged;
pub struct Upload {
    pub staged: Option<Staged>,
    pub size: u64,
    pub digest: Option<String>,
}
impl Upload {
    pub fn empty() -> Self {
        Self {
            staged: None,
            size: 0,
            digest: None,
        }
    }
}
