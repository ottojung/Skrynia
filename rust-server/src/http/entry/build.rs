use std::env;
use std::io;
fn optional(name: &str) -> io::Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value.trim().into())),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(io::Error::other("invalid build info encoding")),
    }
}
fn valid(version: &str, commit: &str) -> bool {
    !version.is_empty()
        && !commit.is_empty()
        && ((version == "development" && commit == "development")
            || (commit.len() >= 40
                && commit.len() <= 64
                && commit.bytes().all(|b| b.is_ascii_hexdigit())))
}
pub fn read() -> io::Result<(String, String)> {
    let version = optional("SKRYNIA_VERSION")?;
    let commit = optional("SKRYNIA_COMMIT")?;
    match (version, commit) {
        (None, None) => Ok(("development".into(), "development".into())),
        (Some(version), Some(commit)) if valid(&version, &commit) => Ok((version, commit)),
        _ => Err(io::Error::other("invalid or incomplete build info")),
    }
}
pub fn info() -> serde_json::Value {
    let (version, commit) = read().expect("validated build metadata");
    serde_json::json!({"version":version,"commit":commit})
}
pub fn validate() -> io::Result<()> {
    read().map(|_| ())
}
#[cfg(test)]
mod tests;
