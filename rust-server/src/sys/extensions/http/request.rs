use std::io::Read;

pub(super) fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .build()
        .map_err(|e| e.to_string())
}

pub(super) fn call(
    client: &reqwest::blocking::Client,
    url: &reqwest::Url,
    arg: u32,
) -> Result<u32, String> {
    let mut response = client
        .post(url.clone())
        .header("Content-Type", "application/octet-stream")
        .body(arg.to_le_bytes().to_vec())
        .send()
        .map_err(|e| e.to_string())?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("syscall provider status: {}", response.status()));
    }
    let mut bytes = [0u8; 4];
    response.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    if response.read(&mut [0u8; 1]).map_err(|e| e.to_string())? != 0 {
        return Err("invalid syscall response length".into());
    }
    Ok(u32::from_le_bytes(bytes))
}
