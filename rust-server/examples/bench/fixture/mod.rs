mod auth;
mod start;
use std::path::PathBuf;
use std::process::Child;
pub struct Target {
    pub base: String,
    pub client: reqwest::Client,
    home: PathBuf,
    child: Child,
}
impl Drop for Target {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}
