use std::collections::BTreeMap;
use std::io::Read;

const MAX_BYTES: u64 = 8 * 1024;

pub(super) fn signatures(
    client: &reqwest::blocking::Client,
    base: &reqwest::Url,
) -> Result<BTreeMap<String, crate::Signature>, String> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| "invalid base URL")?
        .push("syscalls_list");
    let value = get(client, url)?;
    crate::sys::signature::from_discovery(&value)
}

fn get(client: &reqwest::blocking::Client, url: reqwest::Url) -> Result<serde_json::Value, String> {
    let mut response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| e.to_string())?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("syscall discovery status: {}", response.status()));
    }
    read(&mut response)
}

pub(super) fn read(response: &mut impl Read) -> Result<serde_json::Value, String> {
    let mut bytes = Vec::new();
    response
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("discovery response too large".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
