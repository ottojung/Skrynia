mod child;
mod directory;
mod root;
pub fn run() -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!("skrynia-v3-adversarial-{}", std::process::id()));
    if dir.exists() {
        return Err("adversarial test path already exists".into());
    }
    std::fs::create_dir_all(dir.join("data")).map_err(|e| e.to_string())?;
    let results = (|| -> Result<(), String> {
        root::check(&dir)?;
        directory::check(&dir)?;
        child::check(&dir)
    })();
    std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    results
}
