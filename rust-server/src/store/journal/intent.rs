use std::io;
use std::path::Path;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub key: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    pub committed: bool,
}
impl Intent {
    pub fn valid(&self) -> bool {
        crate::valid_key(&self.key)
            && ["create", "replace", "delete"].contains(&self.kind.as_str())
            && if self.kind == "delete" {
                self.sha256.is_none()
            } else {
                self.sha256.as_ref().is_some_and(|x| crate::valid_sha256(x))
            }
    }
    pub fn applied(&self, root: &Path) -> io::Result<bool> {
        let file = root.join("store").join(&self.key);
        if self.kind == "delete" {
            return Ok(!file.exists());
        }
        if !file.exists() {
            return Ok(false);
        }
        Ok(Some(crate::sha_file(&file)?) == self.sha256)
    }
}
