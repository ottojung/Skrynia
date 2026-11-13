use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
pub fn root_credentials(dir: &Path) -> io::Result<Value> {
    let mut found = None;
    for record in fs::read_dir(dir.join("state/grants"))? {
        let path = record?.path();
        let content = fs::read(&path)?;
        let value: Value =
            serde_json::from_slice(&content).map_err(|_| io::Error::other("invalid grant"))?;
        if value.get("parent").is_some_and(Value::is_null) {
            if found.is_some() {
                return Err(io::Error::other("ambiguous root grant"));
            }
            let id = value.get("id").and_then(Value::as_str);
            let token = value.get("rootIssuerToken").and_then(Value::as_str);
            match (id, token) {
                (Some(id), Some(token))
                    if crate::valid_identifier(id) && crate::valid_sha256(token) =>
                {
                    found = Some(serde_json::json!({"id": id, "issuerToken": token}));
                }
                _ => return Err(io::Error::other("invalid root credentials")),
            }
        }
    }
    found.ok_or_else(|| io::Error::other("root grant not found"))
}
pub fn run() -> io::Result<()> {
    let dir = std::env::var("SKRYNIA_DATA_DIR").unwrap_or_else(|_| "/var/lib/skrynia".into());
    let credentials = root_credentials(Path::new(&dir))?;
    println!("{credentials}");
    Ok(())
}
