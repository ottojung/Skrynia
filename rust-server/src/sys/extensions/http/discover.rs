use std::io::Read;

const MAX_VERSION_BYTES: u64 = 8 * 1024;

pub(super) fn names(
    client: &reqwest::blocking::Client,
    base: &reqwest::Url,
) -> Result<std::collections::BTreeSet<String>, String> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| "invalid base URL")?
        .push("version");
    let response = client
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .send()
        .map_err(|e| e.to_string())?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("syscall discovery status: {}", response.status()));
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_VERSION_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_VERSION_BYTES {
        return Err("syscall discovery response too large".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let list = value
        .get("syscalls")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing syscall list")?;
    let mut names = std::collections::BTreeSet::new();
    for item in list {
        let name = item.as_str().ok_or("invalid syscall name type")?;
        super::super::names::check(name)?;
        if name == "version" {
            return Err("reserved syscall name: version".into());
        }
        names.insert(name.to_string());
    }
    Ok(names)
}
