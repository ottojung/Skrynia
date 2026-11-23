mod launch;
#[cfg(test)]
mod smoke;
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
impl Target {
    pub fn verify_write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = std::fs::read(self.home.join("user/bench.dat"))?;
        assert_eq!(bytes, vec![b'A'; 4096]);
        assert!(!self.home.join("operator/bench.dat").exists());
        Ok(())
    }
}
