use std::fs;
use std::path::PathBuf;
pub struct Fixture {
    pub root: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!("skrynia-journal-{}", crate::random_id()));
        for dir in ["store", "store-tmp", "state/store-transactions"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        Self { root }
    }
    pub fn record(&self, id: &str, key: &str, kind: &str, bytes: Option<&[u8]>, committed: bool) {
        if let Some(bytes) = bytes {
            fs::write(self.root.join("store").join(key), bytes).unwrap();
        }
        let file = self
            .root
            .join("state/store-transactions")
            .join(format!("{id}.json"));
        let hash = if kind == "delete" {
            None
        } else {
            Some(crate::sha(b"new"))
        };
        let record = serde_json::json!({"key":key,"kind":kind,"sha256":hash,"committed":committed});
        fs::write(file, serde_json::to_vec(&record).unwrap()).unwrap();
    }
    pub fn replay(&self) -> std::io::Result<usize> {
        let mut calls = 0;
        super::super::super::recover(&self.root, |_intent| {
            calls += 1;
            Ok(())
        })?;
        Ok(calls)
    }
    pub fn transactions(&self) -> usize {
        fs::read_dir(self.root.join("state/store-transactions"))
            .unwrap()
            .count()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
