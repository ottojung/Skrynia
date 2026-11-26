use std::collections::BTreeMap;

/// A provider without POST /version is accepted; a mismatch is rejected.
pub(super) fn cross_check(
    client: &reqwest::blocking::Client,
    base: &reqwest::Url,
    discovered: &BTreeMap<String, crate::Signature>,
) -> Result<(), String> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| "invalid base URL")?
        .push("version");
    let mut response = client
        .post(url)
        .body("")
        .send()
        .map_err(|e| e.to_string())?;
    if response.status() == reqwest::StatusCode::NOT_FOUND
        || response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED
    {
        return Ok(());
    }
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("version status: {}", response.status()));
    }
    let value = super::discover::read(&mut response)?;
    let advertised = crate::sys::signature::from_discovery(&value["syscalls"])?;
    if &advertised != discovered {
        return Err("version/discovery signature mismatch".into());
    }
    Ok(())
}
