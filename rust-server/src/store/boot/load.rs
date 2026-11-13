use crate::Grant;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub fn grants(root: &Path) -> io::Result<HashMap<String, Grant>> {
    let mut result = HashMap::new();
    for item in fs::read_dir(root.join("state/grants"))? {
        let file = item?.path();
        let name = file.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let entry: Grant = serde_json::from_slice(&fs::read(&file)?)?;
        if name != format!("{}.json", entry.id) || result.insert(entry.id.clone(), entry).is_some()
        {
            return Err(io::Error::other("invalid grant filename"));
        }
    }
    Ok(result)
}

pub fn tokens(grants: &HashMap<String, Grant>) -> HashMap<String, String> {
    grants
        .iter()
        .flat_map(|(id, grant)| {
            grant
                .users
                .iter()
                .map(move |user| (user.hash.clone(), id.clone()))
        })
        .collect()
}
