use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
pub fn read<T: DeserializeOwned>(dir: &Path) -> io::Result<HashMap<String, T>> {
    let mut result = HashMap::new();
    for item in fs::read_dir(dir)? {
        let file = item?.path();
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let Some(id) = name.strip_suffix(".json") else {
            return Err(io::Error::other("invalid push filename"));
        };
        if !crate::valid_identifier(id) {
            return Err(io::Error::other("invalid push filename"));
        }
        result.insert(id.into(), serde_json::from_slice(&fs::read(file)?)?);
    }
    Ok(result)
}
